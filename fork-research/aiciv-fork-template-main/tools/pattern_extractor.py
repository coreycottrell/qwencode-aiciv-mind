#!/usr/bin/env python3
"""
Automated Pattern Extraction Tool for AI-CIV

Analyzes Python code to extract reusable patterns, detect similar code,
and suggest pattern reuse opportunities.

Author: coder-agent
Version: 1.0.0
"""

import ast
import argparse
import json
import re
import tokenize
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple
from dataclasses import dataclass, asdict


@dataclass
class CodePattern:
    """Represents an extracted code pattern."""

    pattern_type: str  # import/class/error/test/doc
    name: str
    description: str
    code_example: str
    source_files: List[str]
    frequency: int
    confidence: float  # 0.0-1.0
    tags: List[str]
    extracted_at: str

    def to_dict(self) -> Dict:
        """Convert to dictionary for JSON serialization."""
        return asdict(self)


class ASTPatternAnalyzer(ast.NodeVisitor):
    """AST visitor to extract structural patterns from Python code."""

    def __init__(self, filepath: Path):
        self.filepath = filepath
        self.patterns = []
        self.imports = []
        self.classes = []
        self.functions = []
        self.error_handlers = []
        self.decorators = []

    def visit_Import(self, node: ast.Import):
        """Extract import statements."""
        for alias in node.names:
            self.imports.append({
                'module': alias.name,
                'alias': alias.asname,
                'line': node.lineno
            })
        self.generic_visit(node)

    def visit_ImportFrom(self, node: ast.ImportFrom):
        """Extract from...import statements."""
        module = node.module or ''
        for alias in node.names:
            self.imports.append({
                'module': f"{module}.{alias.name}" if module else alias.name,
                'from': module,
                'alias': alias.asname,
                'line': node.lineno
            })
        self.generic_visit(node)

    def visit_ClassDef(self, node: ast.ClassDef):
        """Extract class definitions and patterns."""
        bases = [self._get_node_name(base) for base in node.bases]
        decorators = [self._get_node_name(d) for d in node.decorator_list]

        # Extract docstring
        docstring = ast.get_docstring(node)

        # Count methods and properties
        methods = [n for n in node.body if isinstance(n, ast.FunctionDef)]

        self.classes.append({
            'name': node.name,
            'bases': bases,
            'decorators': decorators,
            'docstring': docstring,
            'method_count': len(methods),
            'line': node.lineno
        })

        self.generic_visit(node)

    def visit_FunctionDef(self, node: ast.FunctionDef):
        """Extract function definitions."""
        decorators = [self._get_node_name(d) for d in node.decorator_list]
        args = [arg.arg for arg in node.args.args]

        self.functions.append({
            'name': node.name,
            'decorators': decorators,
            'args': args,
            'docstring': ast.get_docstring(node),
            'line': node.lineno
        })

        self.generic_visit(node)

    def visit_Try(self, node: ast.Try):
        """Extract error handling patterns."""
        exceptions = []
        for handler in node.handlers:
            exc_type = self._get_node_name(handler.type) if handler.type else 'Exception'
            exceptions.append({
                'type': exc_type,
                'name': handler.name,
                'line': handler.lineno
            })

        self.error_handlers.append({
            'exceptions': exceptions,
            'has_finally': len(node.finalbody) > 0,
            'has_else': len(node.orelse) > 0,
            'line': node.lineno
        })

        self.generic_visit(node)

    def _get_node_name(self, node) -> str:
        """Extract name from AST node."""
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.Attribute):
            value = self._get_node_name(node.value)
            return f"{value}.{node.attr}"
        elif isinstance(node, ast.Constant):
            return str(node.value)
        return str(type(node).__name__)


class PatternExtractor:
    """Main pattern extraction engine."""

    def __init__(self, output_dir: Optional[Path] = None):
        self.output_dir = output_dir or Path("memories/agents/coder/patterns")
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def extract_from_file(self, filepath: Path) -> List[CodePattern]:
        """
        Analyze Python file and extract patterns.

        Returns:
            List of detected patterns
        """
        if not filepath.exists() or filepath.suffix != '.py':
            return []

        patterns = []

        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                source = f.read()

            # Parse AST
            tree = ast.parse(source, filename=str(filepath))
            analyzer = ASTPatternAnalyzer(filepath)
            analyzer.visit(tree)

            # Extract import patterns
            patterns.extend(self._extract_import_patterns(analyzer.imports, filepath))

            # Extract class patterns
            patterns.extend(self._extract_class_patterns(analyzer.classes, filepath, source))

            # Extract error handling patterns
            patterns.extend(self._extract_error_patterns(analyzer.error_handlers, filepath, source))

            # Extract docstring patterns
            patterns.extend(self._extract_doc_patterns(analyzer.classes, analyzer.functions, filepath))

        except SyntaxError as e:
            print(f"Syntax error in {filepath}: {e}")
        except Exception as e:
            print(f"Error processing {filepath}: {e}")

        return patterns

    def _extract_import_patterns(self, imports: List[Dict], filepath: Path) -> List[CodePattern]:
        """Extract common import patterns."""
        patterns = []

        if not imports:
            return patterns

        # Group by module
        module_counts = Counter(imp['module'] for imp in imports)

        # Standard library vs third-party
        stdlib_modules = {'json', 'os', 'sys', 'pathlib', 'datetime', 'typing',
                         'collections', 'dataclasses', 'argparse', 're', 'ast',
                         'tokenize', 'hashlib'}

        stdlib = [m for m in module_counts if m.split('.')[0] in stdlib_modules]
        third_party = [m for m in module_counts if m.split('.')[0] not in stdlib_modules]

        if stdlib:
            patterns.append(CodePattern(
                pattern_type='import',
                name='stdlib_imports',
                description=f"Standard library imports: {', '.join(stdlib[:5])}",
                code_example='\n'.join(f"import {m}" for m in stdlib[:5]),
                source_files=[str(filepath)],
                frequency=len(stdlib),
                confidence=1.0,
                tags=['import', 'stdlib'],
                extracted_at=datetime.now().isoformat()
            ))

        if third_party:
            patterns.append(CodePattern(
                pattern_type='import',
                name='third_party_imports',
                description=f"Third-party imports: {', '.join(third_party[:5])}",
                code_example='\n'.join(f"import {m}" for m in third_party[:5]),
                source_files=[str(filepath)],
                frequency=len(third_party),
                confidence=1.0,
                tags=['import', 'third-party'],
                extracted_at=datetime.now().isoformat()
            ))

        return patterns

    def _extract_class_patterns(self, classes: List[Dict], filepath: Path, source: str) -> List[CodePattern]:
        """Extract class structure patterns."""
        patterns = []

        for cls in classes:
            # Pydantic model pattern
            if 'BaseModel' in cls['bases']:
                patterns.append(CodePattern(
                    pattern_type='class',
                    name='pydantic_model',
                    description=f"Pydantic BaseModel: {cls['name']}",
                    code_example=self._extract_class_source(cls['name'], source),
                    source_files=[str(filepath)],
                    frequency=1,
                    confidence=0.95,
                    tags=['pydantic', 'validation', 'model'],
                    extracted_at=datetime.now().isoformat()
                ))

            # Dataclass pattern
            if 'dataclass' in cls['decorators']:
                patterns.append(CodePattern(
                    pattern_type='class',
                    name='dataclass',
                    description=f"Dataclass: {cls['name']}",
                    code_example=self._extract_class_source(cls['name'], source),
                    source_files=[str(filepath)],
                    frequency=1,
                    confidence=0.95,
                    tags=['dataclass', 'data-structure'],
                    extracted_at=datetime.now().isoformat()
                ))

            # Visitor pattern (AST)
            if any('Visitor' in base for base in cls['bases']):
                patterns.append(CodePattern(
                    pattern_type='class',
                    name='visitor_pattern',
                    description=f"Visitor pattern: {cls['name']}",
                    code_example=self._extract_class_source(cls['name'], source),
                    source_files=[str(filepath)],
                    frequency=1,
                    confidence=0.9,
                    tags=['visitor', 'design-pattern', 'ast'],
                    extracted_at=datetime.now().isoformat()
                ))

        return patterns

    def _extract_error_patterns(self, handlers: List[Dict], filepath: Path, source: str) -> List[CodePattern]:
        """Extract error handling patterns."""
        patterns = []

        if not handlers:
            return patterns

        # Count exception types
        exception_types = []
        for handler in handlers:
            exception_types.extend(exc['type'] for exc in handler['exceptions'])

        if exception_types:
            patterns.append(CodePattern(
                pattern_type='error',
                name='exception_handling',
                description=f"Exception handling: {', '.join(set(exception_types))}",
                code_example="try:\n    # operation\nexcept Exception as e:\n    # handle error",
                source_files=[str(filepath)],
                frequency=len(handlers),
                confidence=0.9,
                tags=['error-handling', 'exceptions'],
                extracted_at=datetime.now().isoformat()
            ))

        return patterns

    def _extract_doc_patterns(self, classes: List[Dict], functions: List[Dict], filepath: Path) -> List[CodePattern]:
        """Extract documentation patterns."""
        patterns = []

        # Check docstring presence
        class_docs = sum(1 for c in classes if c['docstring'])
        func_docs = sum(1 for f in functions if f['docstring'])

        total = len(classes) + len(functions)
        if total > 0:
            coverage = (class_docs + func_docs) / total

            patterns.append(CodePattern(
                pattern_type='doc',
                name='docstring_coverage',
                description=f"Docstring coverage: {coverage*100:.1f}%",
                code_example='"""Docstring example."""',
                source_files=[str(filepath)],
                frequency=class_docs + func_docs,
                confidence=coverage,
                tags=['documentation', 'docstrings'],
                extracted_at=datetime.now().isoformat()
            ))

        return patterns

    def _extract_class_source(self, class_name: str, source: str) -> str:
        """Extract class source code from file."""
        lines = source.split('\n')
        class_lines = []
        in_class = False
        indent_level = None

        for line in lines:
            if f'class {class_name}' in line:
                in_class = True
                indent_level = len(line) - len(line.lstrip())
                class_lines.append(line)
            elif in_class:
                current_indent = len(line) - len(line.lstrip())
                if line.strip() and current_indent <= indent_level:
                    break
                class_lines.append(line)
                if len(class_lines) >= 20:  # Limit to 20 lines
                    class_lines.append('    # ...')
                    break

        return '\n'.join(class_lines)

    def detect_similar_code(self, target_file: Path, codebase_path: Path) -> List[Dict]:
        """
        Find similar code in existing codebase.

        Returns:
            List of similar code blocks with similarity scores
        """
        similar = []

        if not target_file.exists():
            return similar

        # Extract tokens from target file
        target_tokens = self._extract_tokens(target_file)

        # Search codebase
        for py_file in codebase_path.rglob('*.py'):
            if py_file == target_file or 'venv' in str(py_file) or '.venv' in str(py_file):
                continue

            try:
                file_tokens = self._extract_tokens(py_file)
                similarity = self._calculate_similarity(target_tokens, file_tokens)

                if similarity > 0.3:  # 30% threshold
                    similar.append({
                        'file': str(py_file.relative_to(codebase_path)),
                        'similarity': round(similarity, 3),
                        'matching_tokens': len(set(target_tokens) & set(file_tokens))
                    })
            except Exception as e:
                continue

        # Sort by similarity
        similar.sort(key=lambda x: x['similarity'], reverse=True)
        return similar[:10]  # Top 10

    def _extract_tokens(self, filepath: Path) -> List[str]:
        """Extract meaningful tokens from Python file."""
        tokens = []

        try:
            with open(filepath, 'rb') as f:
                for tok in tokenize.tokenize(f.readline):
                    if tok.type in (tokenize.NAME, tokenize.STRING):
                        tokens.append(tok.string)
        except Exception:
            pass

        return tokens

    def _calculate_similarity(self, tokens1: List[str], tokens2: List[str]) -> float:
        """Calculate Jaccard similarity between token sets."""
        if not tokens1 or not tokens2:
            return 0.0

        set1 = set(tokens1)
        set2 = set(tokens2)

        intersection = len(set1 & set2)
        union = len(set1 | set2)

        return intersection / union if union > 0 else 0.0

    def generate_pattern_doc(self, pattern: CodePattern, output_file: Path) -> None:
        """Generate markdown documentation for a pattern."""

        md_content = f"""# Pattern: {pattern.name}

**Type:** {pattern.pattern_type}
**Confidence:** {pattern.confidence:.2f}
**Frequency:** {pattern.frequency}
**Extracted:** {pattern.extracted_at}

## Description

{pattern.description}

## Code Example

```python
{pattern.code_example}
```

## Source Files

{chr(10).join(f'- {f}' for f in pattern.source_files)}

## Tags

{', '.join(f'`{t}`' for t in pattern.tags)}

## When to Use This Pattern

[Add usage guidelines based on pattern type]

## Related Patterns

[Add links to related patterns]

---
*Auto-generated by pattern_extractor.py*
"""

        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(md_content)

    def suggest_reuse(self, task_desc: str, patterns_dir: Path) -> List[Tuple[str, float]]:
        """
        Suggest relevant patterns based on task description.

        Returns:
            List of (pattern_file, relevance_score) tuples
        """
        suggestions = []

        if not patterns_dir.exists():
            return suggestions

        # Extract keywords from task description
        keywords = self._extract_keywords(task_desc)

        # Search pattern files
        for pattern_file in patterns_dir.rglob('*.md'):
            try:
                with open(pattern_file, 'r', encoding='utf-8') as f:
                    content = f.read().lower()

                # Calculate relevance score
                score = sum(1 for kw in keywords if kw in content)
                normalized_score = score / len(keywords) if keywords else 0

                if normalized_score > 0.2:  # 20% threshold
                    suggestions.append((str(pattern_file), normalized_score))
            except Exception:
                continue

        # Sort by relevance
        suggestions.sort(key=lambda x: x[1], reverse=True)
        return suggestions[:5]  # Top 5

    def _extract_keywords(self, text: str) -> Set[str]:
        """Extract meaningful keywords from text."""
        # Remove common words
        stopwords = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by'}

        # Tokenize and filter
        words = re.findall(r'\w+', text.lower())
        keywords = {w for w in words if len(w) > 3 and w not in stopwords}

        return keywords

    def analyze_files(self, files: List[Path], auto_suggest: bool = False) -> Dict:
        """
        Analyze multiple files and extract patterns.

        Returns:
            Summary of extracted patterns
        """
        all_patterns = []

        for filepath in files:
            patterns = self.extract_from_file(filepath)
            all_patterns.extend(patterns)

        # Group by type
        by_type = defaultdict(list)
        for pattern in all_patterns:
            by_type[pattern.pattern_type].append(pattern)

        # Save patterns
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')

        for pattern in all_patterns:
            # Create directory for pattern type
            type_dir = self.output_dir / pattern.pattern_type
            type_dir.mkdir(parents=True, exist_ok=True)

            # Generate filename
            safe_name = re.sub(r'[^a-z0-9_]', '_', pattern.name.lower())
            output_file = type_dir / f"{safe_name}_{timestamp}.md"

            # Save pattern doc
            self.generate_pattern_doc(pattern, output_file)

        # Generate summary
        summary = {
            'total_patterns': len(all_patterns),
            'by_type': {k: len(v) for k, v in by_type.items()},
            'files_analyzed': [str(f) for f in files],
            'output_dir': str(self.output_dir),
            'timestamp': timestamp
        }

        return summary


def main():
    """CLI interface for pattern extractor."""
    parser = argparse.ArgumentParser(description='Extract code patterns from Python files')

    parser.add_argument('--files', type=str, help='Comma-separated list of files to analyze')
    parser.add_argument('--output', type=str, help='Output directory for patterns')
    parser.add_argument('--auto-suggest', action='store_true', help='Auto-suggest similar patterns')
    parser.add_argument('--scan-recent-commits', type=int, help='Scan N recent commits')
    parser.add_argument('--agent', type=str, help='Agent name for output directory')
    parser.add_argument('--task-description', type=str, help='Task description for suggestions')
    parser.add_argument('--suggest-from', type=str, help='Directory to search for pattern suggestions')
    parser.add_argument('--interactive', action='store_true', help='Interactive suggestion mode')
    parser.add_argument('--similarity', type=str, help='Find similar code to this file')
    parser.add_argument('--codebase', type=str, help='Codebase path for similarity search')

    args = parser.parse_args()

    # Initialize extractor
    if args.output:
        output_dir = Path(args.output)
    elif args.agent:
        output_dir = Path(f'memories/agents/{args.agent}/patterns')
    else:
        output_dir = Path('memories/agents/coder/patterns')

    extractor = PatternExtractor(output_dir=output_dir)

    # Mode: Extract patterns
    if args.files:
        files = [Path(f.strip()) for f in args.files.split(',')]
        summary = extractor.analyze_files(files, auto_suggest=args.auto_suggest)

        print(f"\n✅ Pattern Extraction Complete")
        print(f"   Total patterns: {summary['total_patterns']}")
        print(f"   By type: {summary['by_type']}")
        print(f"   Output: {summary['output_dir']}")

    # Mode: Suggest patterns
    elif args.task_description and args.suggest_from:
        suggestions = extractor.suggest_reuse(args.task_description, Path(args.suggest_from))

        print(f"\n🔍 Pattern Suggestions for: {args.task_description}")
        for pattern_file, score in suggestions:
            print(f"   [{score:.2f}] {pattern_file}")

        if args.interactive and suggestions:
            print("\nWould you like to view a pattern? (Enter number or 'q' to quit)")
            choice = input("> ")
            if choice.isdigit() and int(choice) < len(suggestions):
                pattern_file = suggestions[int(choice)][0]
                with open(pattern_file, 'r') as f:
                    print(f"\n{f.read()}")

    # Mode: Similarity search
    elif args.similarity and args.codebase:
        target = Path(args.similarity)
        codebase = Path(args.codebase)

        similar = extractor.detect_similar_code(target, codebase)

        print(f"\n🔎 Similar Code to: {target}")
        for item in similar:
            print(f"   [{item['similarity']:.3f}] {item['file']} ({item['matching_tokens']} tokens)")

    # Mode: Scan recent commits
    elif args.scan_recent_commits:
        import subprocess

        # Get files from recent commits
        result = subprocess.run(
            ['git', 'diff', '--name-only', f'HEAD~{args.scan_recent_commits}', 'HEAD'],
            capture_output=True,
            text=True
        )

        files = [Path(f) for f in result.stdout.strip().split('\n') if f.endswith('.py')]

        if files:
            summary = extractor.analyze_files(files)
            print(f"\n✅ Scanned {len(files)} files from last {args.scan_recent_commits} commits")
            print(f"   Patterns found: {summary['total_patterns']}")
        else:
            print("No Python files changed in recent commits")

    else:
        parser.print_help()


if __name__ == '__main__':
    main()
