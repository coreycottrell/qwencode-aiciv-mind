# Hybrid Memory Architectures: Vector + Graph + Keyword Search

A comprehensive survey of systems that combine multiple search paradigms — vector embeddings, knowledge graphs, and keyword/BM25 search — to overcome the limitations of any single approach.

---

## Table of Contents

1. [LangChain MultiQueryRetriever](#1-langchain-multiqueryretriever)
2. [Microsoft GraphRAG](#2-microsoft-graphrag)
3. [Mem0](#3-mem0)
4. [Zep / Graphiti](#4-zep--graphiti)
5. [Cognee](#5-cognee)
6. [HybridRAG (Memgraph)](#6-hybridrag-memgraph)
7. [Comparison Summary](#7-comparison-summary)

---

## 1. LangChain MultiQueryRetriever

### What It Does

MultiQueryRetriever is a LangChain component that addresses a fundamental weakness of vector-based retrieval: **phrasing sensitivity**. A single user query, slightly differently worded, can produce drastically different retrieval results due to the nature of embedding-space distance. MultiQueryRetriever solves this by using an LLM to automatically generate multiple alternative versions of the original query, each from a different perspective, then retrieves documents for all of them and returns the deduplicated union.

### Architecture

```
User Query → LLM → [Query 1, Query 2, Query 3, ...]
                                       ↓
              Vector Retriever (run for each query)
                                       ↓
              [Docs_A, Docs_B, Docs_C, ...]
                                       ↓
              Deduplicate & Union → Final Context
```

### Code Example

```python
from langchain.retrievers.multi_query import MultiQueryRetriever
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(temperature=0)
retriever = MultiQueryRetriever.from_llm(
    retriever=vectordb.as_retriever(), llm=llm
)
unique_docs = retriever.invoke("What are the approaches to Task Decomposition?")
```

With a custom prompt for finer control:

```python
from typing import List
from langchain_core.output_parsers import BaseOutputParser
from langchain_core.prompts import PromptTemplate

class LineListOutputParser(BaseOutputParser[List[str]]):
    def parse(self, text: str) -> List[str]:
        return text.strip().split("\n")

QUERY_PROMPT = PromptTemplate(
    input_variables=["question"],
    template="""You are an AI assistant. Generate five different versions of the user question to improve vector DB retrieval.
Original question: {question}"""
)

llm_chain = QUERY_PROMPT | ChatOpenAI(temperature=0) | LineListOutputParser()
retriever = MultiQueryRetriever(
    retriever=vectordb.as_retriever(), llm_chain=llm_chain, parser_key="lines"
)
```

### Pros

- **Automates prompt tuning** — eliminates manual query engineering
- **Overcomes phrasing sensitivity** — reduces failures from minor wording changes
- **Compensates for embedding limitations** — catches documents that a single embedding query might miss
- **Improves recall** — broader, richer document set by querying from multiple angles
- **Simple to integrate** — drop-in replacement for any LangChain retriever

### Cons

- **Higher latency** — requires an LLM call plus multiple retrieval rounds per request
- **Increased token/cost overhead** — consumes more LLM API credits for query generation
- **Potential noise** — poorly generated queries may retrieve irrelevant documents if prompts are not carefully constrained
- **Vector-only at its core** — does not add graph or keyword search; it only improves vector retrieval coverage

### Best Use Case

RAG and QA systems where users phrase identical information needs in highly varied ways, and where standard cosine similarity underperforms on complex or nuanced terminology. Ideal when you need higher recall without manually iterating on prompts, and latency/cost are acceptable tradeoffs.

---

## 2. Microsoft GraphRAG

### What It Does

Microsoft's GraphRAG is a retrieval mechanism that replaces (or supplements) the flat vector store of naive RAG with a **knowledge graph**. It extracts entities and relationships from source documents, constructs a property graph, applies community detection algorithms, and then leverages both local graph traversal and global community summaries during retrieval. This enables it to answer complex multi-hop questions and dataset-wide synthesis queries that vector-only RAG fundamentally cannot handle.

### Architecture

```
Documents → LLM Entity/Relation Extraction → Property Graph
                                                    ↓
                                    Leiden Algorithm → Community Detection
                                                    ↓
                              LLM Community Summaries → Hierarchical Reports
                                                    ↓
              ┌─────────────────────────────────────────────────┐
              │              Retrieval Layer                     │
              │  Local: 1-3 hop graph traversal for specific     │
              │          entity-level facts                      │
              │  Global: Pre-computed community summaries for    │
              │          dataset-wide questions                  │
              │  Hybrid: Vector chunks + graph context merged    │
              └─────────────────────────────────────────────────┘
                                                    ↓
                          Structured, grounded prompt → LLM Generation
```

**Key technical details:**
- Uses the **Leiden algorithm** (or Louvain) for community detection — clusters of densely interconnected nodes
- Generates hierarchical summaries for each community, stored as graph nodes with `summary` and `rating` properties
- During retrieval, traverses from matched entities to their communities and injects top-ranked reports into context

### Pros

- **Multi-hop reasoning** — excels at complex queries requiring traversing entity relationships (~90% accuracy on structured queries vs ~0% for vector-only RAG)
- **Global summarization** — answers dataset-level questions ("What are the key themes?") via community clustering
- **Explainability and provenance** — transparent, auditable citation paths showing exactly which entities and edges formed the answer
- **Hallucination reduction** — constrains LLM outputs to verified, structurally grounded relationships
- **Synthesizes fragmented information** across multiple sources into holistic overviews

### Cons

- **High computational cost** — ingestion and graph construction are significantly more resource-intensive than simple vector embedding
- **Schema/ontology dependency** — requires deliberate entity/relationship modeling; poor design creates noisy "hairball" graphs
- **Entity resolution complexity** — must accurately unify duplicate references or the graph fragments and misses connections
- **Static summaries** — LLM-generated summaries require full reindexing to incorporate new data or reflect updates
- **Scalability challenges** — nodes with thousands of connections can degrade retrieval performance
- **Latency risks** — unoptimized graph traversals can be slow; requires caching and strict hop-depth limits

### Best Use Case

Domains with rich interdependencies and complex relationship networks: legal contracts, research papers, organizational records, supply chain mapping, biotech/pharma (drug discovery, clinical trial matching), fraud detection (AML), and investigative journalism. Any application where structural context and relationship mapping are as critical as semantic text matching.

---

## 3. Mem0

### What It Does

Mem0 operates as a dedicated **"Memory as a Service"** layer between applications and LLMs. It provides a simple API to automatically ingest, index, retrieve, and manage historical context. Instead of embedding memory logic directly into an agent's control flow, Mem0 dynamically prioritizes relevant information, updates conflicting facts, decays outdated memories, and prevents LLM context window overflow by serving only optimized, task-specific context during inference.

### Architecture

Mem0 uses a **hybrid store** combining three pillars across isolated scopes (user, session, agent):

```
              ┌─────────────────────────────────────────────┐
              │                 Mem0 API                     │
              │  Ingest → Index → Retrieve → Manage Memory   │
              └─────────────────────┬───────────────────────┘
                                    ↓
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
  ┌─────▼─────┐            ┌────────▼────────┐          ┌───────▼───────┐
  │  Vector   │            │     Graph       │          │  Key-Value    │
  │  Store    │            │   Database      │          │  (Episodic)   │
  │           │            │                 │          │               │
  │ Semantic  │            │ Relational      │          │ Temporal      │
  │ matching  │            │ relationships   │          │ sequencing    │
  │           │            │ (Pro tier)      │          │ & fast access │
  └───────────┘            └─────────────────┘          └───────────────┘
```

- **Vector Stores (Hippocampus):** Converts text into high-dimensional embeddings for cosine similarity search. Excels at fuzzy, unstructured retrieval.
- **Graph Databases (Association Cortex):** Stores data as nodes and edges to map explicit, directional relationships. Enables multi-hop reasoning. **Gated behind the Pro tier ($249/mo).**
- **Episodic/KV Storage:** Logs time-ordered event sequences for chronological continuity and session history.
- **Self-editing model:** Automatically updates conflicting facts instead of creating duplicates, maintaining a lean memory footprint.

### Pros

- **Automated memory lifecycle** — conflict resolution, decay, utility-based prioritization
- **Decouples memory from agent reasoning** — reduces RAG pipeline development overhead
- **Prevents context window bloat** — mitigates "Lost in the Middle" retrieval failure
- **Enterprise-ready** — managed cloud with SOC 2 Type II / HIPAA compliance
- **Largest community** — ~48K GitHub stars, MCP server integration
- **Plug-and-play** — compatible with OpenAI, Anthropic, LangChain, LlamaIndex

### Cons

- **Graph memory is paywalled** — requires the $249/mo Pro tier
- **No keyword/BM25 search** — relies only on vectors, graph, and KV stores
- **No temporal fact modeling** — lacks validity windows or supersession tracking
- **Multi-agent shared memory** — requires custom work for native support
- **Lower temporal retrieval accuracy** — scored 49.0% on LongMemEval benchmark

### Best Use Case

Long-running conversational systems: customer support (recall of prior tickets, user frustration levels), healthcare (medication schedules, allergy history), education (adaptive tutoring that retains knowledge gaps), sales/CRM (relationship management across stakeholders over weeks/months), and e-commerce personalization. Best for teams that want a managed, plug-and-play memory layer and are willing to pay for graph capabilities.

---

## 4. Zep / Graphiti

### What It Does

Zep is a **temporal knowledge graph** architecture for AI agent memory, built on its open-source engine **Graphiti**. It stores facts as nodes with explicit start/end validity windows, decouples ingestion from retrieval, and tracks entity identity across unstructured and structured data. Zep is the only framework natively engineered to fuse **vector + graph + keyword (BM25)** search in a single retrieval pipeline.

### Architecture

```
              ┌─────────────────────────────────────────────┐
              │                 Zep API                      │
              │  Ingest → Graphiti Engine → Retrieve         │
              └─────────────────────┬───────────────────────┘
                                    ↓
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
  ┌─────▼─────┐            ┌────────▼────────┐          ┌───────▼───────┐
  │  Vector   │            │  Temporal       │          │     BM25      │
  │ Embeddings│            │  Graph          │          │  Keyword      │
  │           │            │  (w/ validity   │          │  Search       │
  │ Semantic  │            │   windows)      │          │               │
  │ matching  │            │                 │          │ Exact text    │
  │           │            │ Multi-hop +     │          │ matching      │
  │           │            │ time-bounded    │          │               │
  └───────────┘            └─────────────────┘          └───────────────┘
                                    ↓
              Deterministic retrieval — no LLM calls at query time
              (~300ms P95 latency)
```

**Subgraph hierarchy (Graphiti):**
- **Episode Subgraph** — per-conversation memory
- **Entity Subgraph** — cross-conversation entity knowledge
- **Global Subgraph** — aggregate knowledge across all episodes

### Pros

- **True hybrid search** — natively combines semantic embeddings, BM25 keyword search, and direct graph traversal
- **No LLM inference at query time** — deterministic retrieval at ~300ms P95 latency
- **Temporal reasoning** — highest accuracy on LongMemEval (63.8%, +15 points vs Mem0)
- **Time-bound facts** — retrieves facts valid at a specific time, not just recent/similar matches
- **Open-source** — Graphiti available for self-hosting
- **Ingests diverse data** — conversation history + structured business JSON

### Cons

- **Managed cloud less polished** than self-hosted version
- **No constitutional/governance validation** for ingested data
- **Opaque enterprise pricing**

### Best Use Case

Applications requiring temporal accuracy and relationship tracking: compliance and audit trails, multi-session conversational agents, business intelligence where facts have validity periods, and any system where knowing "what was true when" matters as much as "what is relevant." Ideal for teams wanting the most accurate temporal knowledge graph with open-source flexibility.

---

## 5. Cognee

### What It Does

Cognee is a **poly-store** hybrid memory framework that combines vector search, pluggable graph database backends, and a relational metadata layer. It runs 100% local/air-gapped via Ollama, making it uniquely suited for privacy-sensitive or offline deployments.

### Architecture

```
              ┌─────────────────────────────────────────────┐
              │              Cognee API                      │
              │  .add() → .cognify() → .search()             │
              └─────────────────────┬───────────────────────┘
                                    ↓
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
  ┌─────▼─────┐            ┌────────▼────────┐          ┌───────▼───────┐
  │  Vector   │            │   Pluggable     │          │  Relational   │
  │  Search   │            │   Graph DB      │          │  Metadata     │
  │           │            │                 │          │  Layer        │
  │ Semantic  │            │ Neo4j / FalkorDB│          │               │
  │ matching  │            │ / KuzuDB /      │          │ Structured    │
  │           │            │  NetworkX       │          │ metadata      │
  └───────────┘            └─────────────────┘          └───────────────┘
                                    ↓
              Background Memify pipeline auto-enriches & prunes stale data
```

### Pros

- **Highly flexible** — swap graph backends without API changes
- **Minimal setup** — three simple API calls: `.add()`, `.cognify()`, `.search()`
- **Fully local deployment** — runs air-gapped via Ollama
- **Background Memify** — auto-enriches and prunes stale data
- **Time Awareness** — new temporal feature for time-bounded facts

### Cons

- **Smaller community** — ~7K stars vs 48K for Mem0
- **No SOC 2 / HIPAA compliance**
- **No native keyword/BM25 search** — relies on underlying DB capabilities
- **Temporal feature is new** — less battle-tested than competitors

### Best Use Case

Privacy-first, offline, or air-gapped deployments where data cannot leave the local environment. Research teams and developers who need flexible backend swapping and minimal setup. Good for prototyping hybrid search without vendor lock-in.

---

## 6. HybridRAG (Memgraph)

### What It Does

HybridRAG, as implemented by Memgraph, is a **dual-database RAG pipeline** that integrates a vector database for semantic similarity matching with a knowledge graph for relational mapping and reasoning. Both systems are orchestrated together to feed context-rich data into an LLM, combining the broad semantic coverage of vectors with the precise relational structure of graphs.

### Architecture

```
User Query
    ↓
┌───────────────────────┐    ┌───────────────────────┐
│   Vector Database     │    │   Knowledge Graph     │
│   Semantic Retrieval  │    │   Graph Reasoning     │
│                       │    │                       │
│ Cosine similarity     │───▶│ Multi-hop traversal   │
│ for concept matching  │    │ for explicit links    │
└───────────────────────┘    └───────────────────────┘
                ↓                    ↓
              Dynamic Synthesis → LLM Generation
```

### Pros

- **Complementary strengths** — merges semantic similarity (vectors) with explicit relational context (graphs)
- **Efficient scaling** — vectors handle unstructured data; graphs handle relationship-heavy structured data
- **Multi-hop reasoning** — uncovers deep, multi-step entity connections that vector search alone cannot explain
- **Adaptive querying** — supports broad-to-narrow search paths, dynamically adjusts based on query ambiguity
- **Real-time + historical context** — combines live graph updates with stable historical vector embeddings

### Cons

- **Dual-system complexity** — requires maintaining and synchronizing two separate data stores
- **Graph construction overhead** — same challenges as GraphRAG (entity extraction, schema design)
- **Orchestration complexity** — deciding when to use vector vs graph vs both requires careful engineering

### Best Use Case

Scientific research (drug discovery, biomedical knowledge bases like Cedars-Sinai's Alzheimer's Knowledge Base), chronic disease management, financial analysis (mapping partnerships and supply chains), and enterprise AI applications with complex compound queries requiring both semantic breadth and relational depth.

---

## 7. Comparison Summary

| Feature | MultiQueryRetriever | GraphRAG | Mem0 | Zep/Graphiti | Cognee | HybridRAG |
|---|---|---|---|---|---|---|
| **Vector Search** | Yes | Yes (hybrid) | Yes | Yes | Yes | Yes |
| **Graph Search** | No | Yes | Yes (Pro) | Yes | Yes | Yes |
| **Keyword/BM25** | No | No | No | Yes | No | No |
| **Temporal Facts** | No | No | No | Yes | Yes (new) | No |
| **LLM at Query** | Yes (generates queries) | No | No | No | No | No |
| **Open Source** | Yes | Yes | Partial | Yes (Graphiti) | Yes | Yes |
| **Managed Cloud** | N/A | N/A | Yes (SOC 2/HIPAA) | Yes | No | N/A |
| **Local/Air-gapped** | Yes | Yes | No | Self-host | Yes | Yes |
| **Multi-hop Reasoning** | No | Yes | Limited | Yes | Yes | Yes |
| **Best For** | Improving vector recall | Complex relationship queries | Conversational memory | Temporal knowledge graphs | Privacy-first deployments | Scientific research |

### Key Takeaway

No single system perfectly unifies all three paradigms (vector + graph + keyword) without tradeoffs:

- **Zep/Graphiti** is the only framework with native vector + graph + BM25 integration, plus temporal fact modeling
- **Microsoft GraphRAG** is the strongest for graph-centric reasoning with community-level summarization
- **Mem0** offers the most polished managed experience but gates graph capabilities behind a paywall and lacks keyword search
- **MultiQueryRetriever** is a clever technique for improving vector recall but does not add graph or keyword capabilities
- **Cognee** leads on deployment flexibility with pluggable backends and full local operation
- **HybridRAG** provides a clean dual-database pattern well-suited for scientific and enterprise use cases

The choice depends on whether your priority is **temporal accuracy** (Zep), **relationship depth** (GraphRAG), **managed simplicity** (Mem0), **deployment control** (Cognee), or **improved vector recall** (MultiQueryRetriever).
