#!/usr/bin/env python3
"""SimpleMem Hybrid Search for Hengshi (衡实) — M5

Adapted from arXiv:2604.01007 Omni-SimpleMem paper.
Hybrid Dense-Sparse search with SET-UNION merging (NOT score fusion).

Architecture:
  Dense: numpy cosine similarity over text embeddings (no FAISS needed at our scale)
  Sparse: ripgrep BM25-like keyword search over memory files
  Merge: R(q) = D(q) union (K(q) \\ D(q))  -- dense keeps ranking, sparse-only appended

Pyramid Retrieval:
  Level 1 - Preview: summaries only (~10 tokens each)
  Level 2 - Details: full text (if similarity > theta=0.4)
  Level 3 - Evidence: raw content (budget B=6000 tokens)
"""
import json
import math
import re
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import numpy as np

# ──────────────────────────────────────────────────────────────
# Embedding — lightweight sentence encoding (no external model)
# Uses TF-IDF + SVD approximation for dense vectors
# ──────────────────────────────────────────────────────────────

class SimpleEmbedder:
    """Lightweight text embedder using character n-gram TF-IDF + random projection.
    
    At our scale (hundreds of memories), this beats waiting for a model API call.
    For production: replace with ollama embeddings or a local model.
    """
    
    def __init__(self, n_features: int = 512, ngram_range: tuple = (1, 3)):
        self.n_features = n_features
        self.ngram_range = ngram_range
        self._vocab: dict[str, int] = {}
        self._idf: np.ndarray = np.ones(n_features)  # Start with uniform, not zeros
        self._doc_count = 0
        self._trained = False

    def _get_ngrams(self, text: str) -> list[str]:
        """Extract character n-grams from text.

        For Chinese text, character n-grams (1-3) work better than byte-level.
        Each Chinese character is a single codepoint, so char n-grams naturally
        capture semantic units: 深, 度, 学, 深度, 度学, 学习, etc.
        """
        text = text.lower()
        ngrams = []
        for n in range(self.ngram_range[0], self.ngram_range[1] + 1):
            for i in range(len(text) - n + 1):
                ngrams.append(text[i:i+n])
        return ngrams
    
    def _hash_ngram(self, ngram: str) -> int:
        """Simple hash to feature index."""
        h = 0
        for c in ngram:
            h = (h * 31 + ord(c)) % self.n_features
        return h
    
    def fit(self, documents: list[str]):
        """Build vocabulary and IDF weights from documents."""
        self._doc_count = len(documents)

        # BUG-011 FIX: Guard against empty document list
        if not documents:
            self._idf = np.ones(self.n_features)
            self._trained = True
            return

        doc_freq = np.zeros(self.n_features)
        
        for doc in documents:
            ngrams = self._get_ngrams(doc)
            seen = set()
            for ng in ngrams:
                idx = self._hash_ngram(ng)
                if idx not in seen:
                    doc_freq[idx] += 1
                    seen.add(idx)
        
        # BUG-006 FIX: Use standard smoothed IDF formula
        # log((N + 1) / (df + 1)) + 1  (sklearn-style smoothing)
        # Prevents division-by-zero and assigns lower weights to rare features
        self._idf = np.log((self._doc_count + 1) / (doc_freq + 1)) + 1
        self._trained = True
    
    def embed(self, text: str) -> np.ndarray:
        """Embed a single text into dense vector."""
        if not self._trained:
            raise RuntimeError("Embedder not trained. Call fit() first.")
        
        ngrams = self._get_ngrams(text)
        vec = np.zeros(self.n_features)
        for ng in ngrams:
            idx = self._hash_ngram(ng)
            vec[idx] += 1
        
        # TF-IDF weighting
        vec = vec * self._idf
        
        # L2 normalize
        norm = np.linalg.norm(vec)
        if norm > 0:
            vec = vec / norm
        
        return vec
    
    def embed_batch(self, texts: list[str]) -> np.ndarray:
        """Embed multiple texts into matrix."""
        return np.array([self.embed(t) for t in texts])


# ──────────────────────────────────────────────────────────────
# Memory document — each memory file
# ──────────────────────────────────────────────────────────────

@dataclass
class MemoryDoc:
    id: str
    title: str
    content: str
    category: str = ""
    tier: str = "working"
    depth_score: float = 0.0
    file_path: str = ""
    embedding: Optional[np.ndarray] = None
    
    @property
    def summary(self) -> str:
        """First 100 chars of content as preview."""
        return self.content[:100].strip()
    
    @property
    def preview_tokens(self) -> int:
        """Estimated token count of summary."""
        return len(self.summary) // 4


# ──────────────────────────────────────────────────────────────
# Sparse Search — ripgrep keyword matching
# ──────────────────────────────────────────────────────────────

def sparse_search(query: str, search_dir: Path, limit: int = 10) -> list[str]:
    """Search memory files using ripgrep with multi-word query support."""
    try:
        # Split query into words for better matching
        words = query.split()
        if len(words) > 1:
            # Use OR pattern for multi-word queries
            pattern = "|".join(re.escape(w) for w in words)
            result = subprocess.run(
                ["rg", "-l", "-i", "--glob", "*.md", "-e", pattern, str(search_dir)],
                capture_output=True, text=True, timeout=5
            )
        else:
            result = subprocess.run(
                ["rg", "-l", "-i", "--glob", "*.md", query, str(search_dir)],
                capture_output=True, text=True, timeout=5
            )
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip().split("\n")[:limit]
    except (FileNotFoundError, subprocess.TimeoutExpired):
        # Fallback: Python string search
        files = []
        for f in search_dir.rglob("*.md"):
            if query.lower() in f.read_text().lower():
                files.append(str(f))
                if len(files) >= limit:
                    break
        return files
    return []


# ──────────────────────────────────────────────────────────────
# Hybrid Search — SET-UNION merge (SimpleMem pattern)
# R(q) = D(q) ∪ (K(q) \ D(q))
# ──────────────────────────────────────────────────────────────

@dataclass
class SearchResult:
    doc: MemoryDoc
    score: float
    source: str  # "dense" | "sparse" | "both"
    level: int = 1  # Pyramid level 1/2/3


class SimpleMemSearch:
    """Hybrid dense+sparse memory search.
    
    Pattern from Omni-SimpleMem paper:
    1. Dense retrieval: cosine similarity over embeddings
    2. Sparse retrieval: keyword search via ripgrep
    3. Set-union merge: dense results keep original ranking, sparse-only appended
    4. Pyramid retrieval: token-budget-aware expansion
    """
    
    def __init__(self, memory_dir: Path, embedder: SimpleEmbedder = None):
        self.memory_dir = memory_dir
        self.embedder = embedder or SimpleEmbedder()
        self.documents: list[MemoryDoc] = []
        self._index_built = False
    
    def index(self) -> int:
        """Load all memory files and build dense index."""
        self.documents = []
        md_files = list(self.memory_dir.rglob("*.md"))
        
        for f in md_files:
            content = f.read_text()
            # Parse front matter
            title = ""
            body = content
            if "---" in content:
                parts = content.split("---", 2)
                if len(parts) >= 3:
                    header = parts[1]
                    body = parts[2]
                    for line in header.strip().split("\n"):
                        if ": " in line:
                            k, v = line.split(": ", 1)
                            if k.strip() == "title":
                                title = v.strip()
            
            doc = MemoryDoc(
                id=f.stem,
                title=title or f.stem,
                content=body.strip(),
                file_path=str(f),
            )
            self.documents.append(doc)
        
        # Build embeddings
        if self.documents:
            texts = [d.summary + " " + d.title for d in self.documents]
            self.embedder.fit(texts)
            embeddings = self.embedder.embed_batch(texts)
            for doc, emb in zip(self.documents, embeddings):
                doc.embedding = emb
        
        self._index_built = True
        return len(self.documents)
    
    def search(self, query: str, top_k: int = 10, token_budget: int = 6000) -> list[SearchResult]:
        """Hybrid search: dense + sparse, set-union merge, pyramid retrieval."""
        if not self._index_built:
            self.index()
        
        if not self.documents:
            return []
        
        # ── Dense search ──
        query_emb = self.embedder.embed(query)
        doc_embs = np.array([d.embedding for d in self.documents if d.embedding is not None])
        
        if len(doc_embs) > 0:
            similarities = doc_embs @ query_emb  # Cosine sim (vectors are L2-normalized)
            dense_indices = np.argsort(similarities)[::-1][:top_k]
            dense_docs = [(self.documents[i], float(similarities[i])) for i in dense_indices if similarities[i] > 0.1]
        else:
            dense_docs = []
        
        # ── Sparse search ──
        sparse_files = sparse_search(query, self.memory_dir, limit=top_k)
        sparse_ids = {Path(f).stem for f in sparse_files}
        
        # ── Set-union merge (SimpleMem pattern) ──
        # D(q) = dense results, K(q) = sparse results
        # R(q) = D(q) ∪ (K(q) \ D(q))
        # Dense keeps original ranking, sparse-only appended
        
        dense_ids = {d.id for d, _ in dense_docs}
        results = []
        
        # 1. Dense results first (preserve ranking)
        for doc, score in dense_docs:
            source = "both" if doc.id in sparse_ids else "dense"
            results.append(SearchResult(doc=doc, score=score, source=source))
        
        # 2. Sparse-only results appended
        for f in sparse_files:
            doc_id = Path(f).stem
            if doc_id not in dense_ids:
                doc = next((d for d in self.documents if d.id == doc_id), None)
                if doc:
                    results.append(SearchResult(doc=doc, score=0.0, source="sparse"))
        
        # ── Pyramid retrieval ──
        used_tokens = 0
        theta = 0.4  # Similarity threshold for level 2
        budget = token_budget
        
        for r in results:
            preview_tokens = r.doc.preview_tokens
            if used_tokens + preview_tokens > budget:
                break
            
            r.level = 1
            used_tokens += preview_tokens
            
            # Level 2: if similarity > θ, include full content
            if r.score > theta:
                full_tokens = len(r.doc.content) // 4
                if used_tokens + full_tokens <= budget:
                    r.level = 2
                    used_tokens += full_tokens
        
        return results
    
    def search_preview(self, query: str, top_k: int = 5) -> list[str]:
        """Level 1 only: summaries only, minimal tokens."""
        results = self.search(query, top_k=top_k, token_budget=500)
        lines = []
        for r in results:
            source_icon = {"dense": "🔵", "sparse": "🟡", "both": "🟢"}[r.source]
            lines.append(f"{source_icon} [{r.score:.3f}] {r.doc.title}")
            lines.append(f"   {r.doc.summary[:80]}")
        return lines


# ──────────────────────────────────────────────────────────────
# CLI — test it
# ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys
    
    memory_dir = Path("/home/corey/projects/AI-CIV/qwen-aiciv-mind/minds")
    
    # Test 1: Civilizational memory
    civ_dir = memory_dir / "minds" / "_civilizational"
    if civ_dir.exists():
        print("=== Hybrid Search: Civilizational Memory ===")
        search = SimpleMemSearch(civ_dir)
        n = search.index()
        print(f"Indexed {n} documents")
        
        for query in ["memory architecture", "delegation rules", "API usage"]:
            print(f"\nQuery: '{query}'")
            results = search.search(query, top_k=5)
            for r in results:
                print(f"  {r.source:6s} [{r.score:.3f}] L{r.level} {r.doc.title}")
            if not results:
                print("  (no results)")
    
    # Test 2: Battle test results
    print("\n=== Hybrid Search: Battle Test Memory ===")
    bt_dir = memory_dir / "battle_test"
    if bt_dir.exists():
        search = SimpleMemSearch(bt_dir)
        n = search.index()
        print(f"Indexed {n} documents")
        
        results = search.search("delegation chain", top_k=5)
        for r in results:
            print(f"  {r.source:6s} [{r.score:.3f}] L{r.level} {r.doc.title}")
