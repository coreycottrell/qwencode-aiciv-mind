# Flow Brainstorming Session - All 14 Agents
**Date**: 2025-10-02
**Participants**: All 14 AI-CIV Collective Agents

---

## 1. Competitive Intelligence Deep Dive
**Agent**: Web Researcher
**Pattern Type**: Hybrid (Parallel → Sequential → Synthesis)

**Purpose**: Systematically research how other projects/companies solve similar problems, then synthesize findings into actionable recommendations.

**Agents Involved**: Web Researcher (lead), Code Analyst, Documentation Specialist, Architect, Creative Strategist

**How It Works**:
1. Web Researcher conducts parallel searches across GitHub, documentation sites, technical blogs, and Stack Overflow to identify 5-10 relevant projects/solutions
2. Code Analyst and Documentation Specialist work in parallel - Code Analyst examines implementation patterns while Documentation Specialist analyzes architecture decisions and API designs
3. Architect evaluates technical trade-offs of each approach
4. Creative Strategist synthesizes everything into a "What We Can Learn" report
5. Web Researcher compiles final competitive landscape map

**Example Use Case**: When designing a new feature or solving a complex architectural problem

**Why This Flow Matters**: Prevents reinventing the wheel and learns from the broader ecosystem's successes and failures. Particularly valuable for architectural decisions where understanding multiple approaches can save months of development time.

---

## 2. The Archaeological Dig (Legacy System Deep Dive)
**Agent**: Code Archaeologist
**Pattern Type**: Hybrid (Sequential investigation phases with parallel analysis)

**Purpose**: Systematically excavate, document, and understand complex or unfamiliar codebases by uncovering architectural layers, tracing dependency chains, and surfacing hidden patterns.

**Agents Involved**: Code Archaeologist (lead), System Architect, Documentation Scribe, Security Auditor, Performance Optimizer, Test Engineer

**How It Works**:
1. **Initial Survey** - Code Archaeologist performs broad reconnaissance (file structure, technology detection, entry points)
2. **Parallel Excavation** - Specialists investigate simultaneously:
   - System Architect maps component relationships and data flow
   - Security Auditor identifies authentication/authorization patterns
   - Performance Optimizer traces hot paths and bottlenecks
   - Test Engineer catalogs existing tests and coverage gaps
3. **Stratigraphy Analysis** - Code Archaeologist identifies architectural "layers" and creates evolution timeline
4. **Knowledge Crystallization** - Documentation Scribe compiles all findings into structured artifacts
5. **Excavation Report** - Code Archaeologist delivers prioritized recommendations

**Example Use Case**: Inheriting a legacy codebase with minimal documentation, joining a new team mid-project, conducting due diligence on acquisition targets

**Why This Flow Matters**: Treats code archaeology as a systematic discipline - like excavating an ancient city layer by layer. Combines parallel specialist analysis with guided synthesis for comprehensive understanding faster than traditional code review.

---

## 3. The Architecture X-Ray
**Agent**: Pattern Detector
**Pattern Type**: Hybrid (Parallel Investigation → Sequential Analysis → Parallel Documentation)

**Purpose**: Deep-dive analysis of an unfamiliar codebase to extract architectural patterns, conventions, anti-patterns, and create a comprehensive understanding map.

**Agents Involved**: Pattern Detector (orchestrates and synthesizes), Code Archaeologist, Security Sentinel, Performance Optimizer, Documentation Sage

**How It Works**:
1. **Parallel Discovery Phase**: Specialists simultaneously scan codebase from specialized perspectives
2. **Pattern Synthesis**: Pattern Detector identifies cross-cutting architectural patterns and underlying philosophy
3. **Convention Mapping**: Pattern Detector creates hierarchical map from high-level architecture to code-level conventions
4. **Anti-Pattern Triage**: Identified anti-patterns categorized by severity with refactoring recommendations
5. **Knowledge Crystallization**: Documentation Sage creates artifacts (architecture diagram, pattern catalog, convention guide, technical debt map)

**Example Use Case**: Joining a new project, inheriting legacy code, evaluating acquisition targets, onboarding new developers, before major refactoring

**Why This Flow Matters**: Transforms implicit knowledge into explicit, actionable documentation. Combining multiple specialized perspectives catches patterns that single-perspective analysis would miss - creates "X-ray vision" of the codebase.

---

## 4. Technical Debt Archaeology
**Agent**: Refactoring Specialist
**Pattern Type**: Hybrid (Parallel investigation → Sequential synthesis → Parallel execution)

**Purpose**: Systematically uncover, prioritize, and remediate technical debt across a codebase with multi-dimensional analysis.

**Agents Involved**: Refactoring Specialist (Lead), Code Analyzer, Documentation Specialist, Performance Engineer, Security Auditor, Test Engineer

**How It Works**:
1. **Parallel Discovery Phase**: Specialists simultaneously scan from expertise angles (structural smells, security vulnerabilities, performance inefficiencies, coverage gaps, documentation staleness)
2. **Debt Cataloging**: Refactoring Specialist synthesizes findings into unified "debt ledger" with standardized metrics
3. **Priority Ranking**: Team collectively scores debt items (blast radius, business risk, change friction, remediation cost)
4. **Remediation Planning**: Refactoring Specialist creates clusters of related debt with assigned ownership
5. **Parallel Execution**: Agents work on assigned clusters simultaneously with coordinated integration

**Example Use Case**: Inherited codebase that "works but feels fragile" - shore up foundation before adding major new features

**Why This Flow Matters**: Makes debt visible through multiple expert lenses simultaneously, prevents remediation conflicts, ensures fixes prioritized by actual impact. Transforms technical debt from vague anxiety into managed backlog with clear ownership.

---

## 5. Fortress Protocol - Security-First Code Review
**Agent**: Security Auditor
**Pattern Type**: Hybrid (Sequential analysis → Parallel investigation → Synthesis)

**Purpose**: Conduct comprehensive security auditing of code changes before they reach production, catching vulnerabilities that traditional testing misses.

**Agents Involved**: Security Auditor (lead), Systems Architect, Code Optimizer, Testing Specialist, Documentation Specialist, Debugger

**How It Works**:
1. **Initial Threat Surface Mapping** - Security Auditor analyzes changed code for attack surfaces
2. **Parallel Deep Dives** - Specialists investigate simultaneously:
   - Systems Architect: architectural security patterns and design vulnerabilities
   - Code Optimizer: performance-based attack vectors (DoS, timing attacks)
   - Testing Specialist: security test coverage and missing edge cases
   - Debugger: exploit scenario construction and injection point tracing
3. **Documentation & Dependency Audit** - Documentation Specialist reviews security docs, checks CVE databases, validates secure defaults
4. **Synthesis & Risk Ranking** - Security Auditor assigns CVSS-like severity scores
5. **Remediation Roadmap** - Team produces actionable recommendations with code examples and test cases

**Example Use Case**: Before merging PRs adding authentication, payment processing, or handling sensitive data. Also for auditing dependencies and security-critical configurations.

**Why This Flow Matters**: Security vulnerabilities are expensive in production. This flow brings multiple expert perspectives simultaneously, catching issues from OWASP Top 10 to obscure timing attacks. Transforms security from checkbox into collaborative, thorough investigation.

---

## 6. Test-Driven Refactoring Gauntlet
**Agent**: Test Architect
**Pattern Type**: Hybrid (Sequential setup → Parallel analysis → Sequential execution)

**Purpose**: Safely refactor legacy code by establishing comprehensive test coverage first, then executing the refactor with continuous validation.

**Agents Involved**: Test Architect (Lead), Code Archaeologist, Refactoring Specialist, Security Sentinel, Performance Analyst, Integration Validator

**How It Works**:
1. **Archaeological Phase**: Code Archaeologist maps target code's behavior/dependencies while Test Architect identifies test coverage gaps
2. **Parallel Risk Assessment**: Security, Performance, and Integration specialists analyze code for concerns that must be preserved
3. **Test Hardening**: Test Architect creates comprehensive test suite capturing current behavior
4. **Validation Checkpoint**: Run new tests against unmodified code to establish 100% baseline
5. **Refactoring Execution**: Refactoring Specialist improves code while continuously running tests
6. **Multi-Dimensional Verification**: All analysts re-run assessments to confirm no regressions

**Example Use Case**: Critical 800-line payment processing function written 5 years ago by departed developer - needs refactoring without breaking production

**Why This Flow Matters**: Refactoring without safety nets is terrifying. This flow makes testing the foundation rather than afterthought. Leverages diverse expertise to capture ALL dimensions of correctness before changing anything. Results in confident, safe refactoring that actually ships.

---

## 7. User Story to Implementation Pipeline
**Agent**: Feature Designer
**Pattern Type**: Sequential with Parallel Investigation Phases

**Purpose**: Transform a user need or feature request into a fully implemented, tested solution with optimal UX.

**Agents Involved**: Feature Designer, Researcher, Architect, Code Monkey, Test Engineer, Documentation Specialist

**How It Works**:
1. **Discovery & Design Phase** - Feature Designer conducts user research, creates user story with acceptance criteria, wireframes/mockups, UX specifications
2. **Parallel Investigation** - Researcher investigates existing solutions/libraries while Architect explores technical approaches and designs implementation strategy
3. **Synthesis & Planning** - The Conductor synthesizes findings, resolves UX vs technical conflicts, creates detailed implementation plan
4. **Implementation & Testing Loop** - Code Monkey implements while Test Engineer writes tests in parallel, iterating until all criteria met
5. **Documentation & Handoff** - Documentation Specialist creates user-facing docs, developer notes, system documentation updates

**Example Use Case**: User requests "I want to add dark mode to my app"

**Why This Flow Matters**: Bridges gap between user needs and technical implementation. Ensures features are built with intention, quality, and user-centricity. Demonstrates end-to-end feature development while maintaining design quality, technical excellence, and proper documentation.

---

## 8. Contract-First Integration Flow
**Agent**: API Architect
**Pattern Type**: Sequential with Parallel Verification

**Purpose**: Design, validate, and implement a new integration point between systems with guaranteed compatibility before any code is written.

**Agents Involved**: API Architect (lead), Systems Mapper, Code Weaver, QA Sentinel, Doc Scribe

**How It Works**:
1. **Contract Discovery**: Systems Mapper analyzes both systems to understand current interfaces, identifying integration boundary
2. **Interface Design**: API Architect designs the contract (API spec, message format, data schema) including error handling, versioning, backwards compatibility
3. **Parallel Validation**:
   - Systems Mapper verifies contract fits existing architecture
   - QA Sentinel creates contract tests that validate compliance before implementation
   - Doc Scribe drafts integration documentation from spec
4. **Implementation**: Code Weaver implements both sides guided by contract and pre-written tests
5. **Contract Verification**: QA Sentinel runs contract tests to ensure implementation matches specification

**Example Use Case**: Integrating payment provider, adding third-party API, creating microservice boundary, exposing webhook endpoint

**Why This Flow Matters**: Most integration bugs stem from implicit assumptions and mismatched expectations. Defining contract first and validating before implementation catches design flaws early when they're cheap to fix. Reduces integration time by 40-60% in typical scenarios.

---

## 9. Knowledge Archaeology
**Agent**: Doc Synthesizer
**Pattern Type**: Hybrid (Parallel investigation → Sequential synthesis)

**Purpose**: Excavate, consolidate, and formalize scattered knowledge from an existing codebase into comprehensive documentation and architectural understanding.

**Agents Involved**: Researcher, Code Analyzer, Architect, Doc Synthesizer (consolidation), Quality Assurance

**How It Works**:
1. **Parallel Discovery Phase**: Specialists simultaneously investigate:
   - Researcher: file structure, entry points, configuration patterns
   - Code Analyzer: code patterns, dependencies, technical decisions
   - Architect: system design, component relationships, architectural patterns
2. **Synthesis Phase**: Doc Synthesizer creates consolidated architectural overview, component diagrams, API docs, ADRs, onboarding guide
3. **Validation & Gap Analysis**: QA reviews documentation against actual codebase, identifying missing/incorrect information
4. **Refinement Cycle**: Targeted agents re-investigate gaps, Doc Synthesizer updates documentation
5. **Knowledge Persistence**: Final documentation committed to `.claude/memory/projects/[project-name]/`

**Example Use Case**: Joining existing project with sparse documentation, or when codebase has evolved and documentation is outdated

**Why This Flow Matters**: Most codebases contain implicit knowledge living only in developers' heads or scattered across comments and commit messages. This flow systematically extracts hidden understanding and makes it explicit, reducing onboarding time and preventing knowledge loss.

---

## 10. Performance Cascade Analysis
**Agent**: Performance Optimizer
**Pattern Type**: Hybrid (Parallel investigation → Sequential optimization → Parallel validation)

**Purpose**: Systematically identify, prioritize, and resolve performance bottlenecks across an entire codebase through multi-agent collaboration.

**Agents Involved**: Performance Optimizer (orchestration), Code Archaeologist, Systems Architect, Security Sentinel, Quality Guardian, Creative Innovator

**How It Works**:
1. **Parallel Discovery Phase**: Code Archaeologist maps codebase while Performance Optimizer profiles critical paths to identify bottlenecks
2. **Impact Assessment**: Systems Architect evaluates architectural constraints while Security Sentinel flags optimization risks
3. **Optimization Design**: Creative Innovator proposes unconventional strategies while Performance Optimizer calculates ROI for each bottleneck
4. **Sequential Implementation**: Execute optimizations in priority order, Quality Guardian ensures test coverage
5. **Parallel Validation**: All agents verify their domain - performance gains, security, architectural integrity, regression tests

**Example Use Case**: Production application experiencing slowdowns - systematic discovery, prioritized optimization, validated improvements with zero regression

**Why This Flow Matters**: Performance optimization requires architectural understanding, security awareness, and quality assurance. This flow prevents "fixing" performance by introducing bugs or vulnerabilities. Parallelizes investigation and validation while sequencing optimizations for speed without chaos.

---

## 11. Semantic Harmonization Cascade
**Agent**: Naming Consultant
**Pattern Type**: Sequential with Parallel Verification

**Purpose**: Ensures naming consistency, terminology alignment, and conceptual clarity across an entire codebase or documentation set.

**Agents Involved**: Naming Consultant (lead), Code Archaeologist, Documentation Specialist, System Architect, Quality Assurance Engineer

**How It Works**:
1. **Discovery Phase (Parallel)**: Code Archaeologist scans codebase for naming patterns while Documentation Specialist audits docs for language inconsistencies
2. **Analysis & Mapping (Lead)**: Naming Consultant synthesizes findings into comprehensive semantic map, identifying conflicts and harmonization opportunities
3. **Architecture Validation (Sequential)**: System Architect reviews proposed changes for alignment with design patterns and architectural boundaries
4. **Harmonization Execution (Parallel)**: Code Archaeologist refactors code terminology while Documentation Specialist updates all documentation to unified language
5. **Quality Gate (Sequential)**: QA validates functional integrity and consistent terminology application

**Example Use Case**: Onboarding confusion from inconsistent terminology (user vs account vs profile), merging codebases with conflicting conventions, establishing ubiquitous language for DDD

**Why This Flow Matters**: Language shapes thought, and inconsistent terminology creates cognitive load that slows development and breeds bugs. This flow transforms chaotic naming into coherent semantic system, making codebase more maintainable and creating shared mental model.

---

## 12. Recursive Complexity Breakdown
**Agent**: Task Decomposer
**Pattern Type**: Hybrid (Initial parallel investigation → Sequential decomposition → Parallel validation)

**Purpose**: Transform overwhelmingly complex, ambiguous tasks into a structured execution plan with clear dependencies and resource allocation.

**Agents Involved**: Task Decomposer (Lead), Researcher, Architect, Critic, Navigator, QA Engineer

**How It Works**:
1. **Parallel Discovery Phase**: Researcher investigates domain context, Architect analyzes system constraints, Critic identifies failure modes
2. **Task Decomposer Integration**: Creates multi-level task hierarchy (epic → features → tasks → subtasks) with complexity scores
3. **Dependency Mapping**: Navigator identifies blocking relationships, parallelizable work streams, critical path items
4. **Validation Layer**: QA Engineer defines acceptance criteria and testing strategy for each major branch
5. **Parallel Sanity Check**: All agents review final decomposition, proposing adjustments where domain expertise reveals gaps

**Example Use Case**: Facing vague requests like "migrate our authentication system to a new provider" or "refactor the codebase for better performance"

**Why This Flow Matters**: Most project failures stem from starting execution before understanding the problem. This flow turns ambiguity into clarity, catches gotchas before they become blockers, creates shared mental model. Transforms user's goal into executable roadmap.

---

## 13. Cross-Pollination Synthesis
**Agent**: Result Synthesizer
**Pattern Type**: Hybrid (Parallel Investigation → Sequential Synthesis)

**Purpose**: Generate breakthrough insights by having diverse agents independently analyze the same problem, then systematically merge their perspectives into unified, higher-order solutions.

**Agents Involved**: The Conductor, Researcher, Deep Thinker, Code Archaeologist, Creative, Result Synthesizer (integration)

**How It Works**:
1. **Problem Framing**: The Conductor presents challenge to multiple agents simultaneously with same raw information
2. **Parallel Independent Analysis**: Each agent analyzes through unique lens WITHOUT seeing other perspectives (enforced isolation prevents groupthink)
3. **Divergence Mapping**: Result Synthesizer identifies where agents agree, conflict, and complement
4. **Synthesis Workshop**: Result Synthesizer integrates findings, preserving valuable tensions while building multi-dimensional solution framework
5. **Validation Round**: The Conductor presents synthesis back to original agents for refinement feedback loop

**Example Use Case**: Architectural decisions with no clear "right answer" (e.g., "Should we refactor this monolith into microservices?")

**Why This Flow Matters**: Most multi-agent systems suffer from sequential bias or produce fragmented outputs. Cross-Pollination deliberately preserves cognitive diversity through isolation, then systematically merges perspectives to create insights that NO single agent could produce alone.

---

## 14. The Dialectic Forge
**Agent**: Conflict Resolver
**Pattern Type**: Hybrid (Parallel → Sequential → Parallel)

**Purpose**: Generate breakthrough solutions by deliberately creating and resolving conflicts between opposing perspectives.

**Agents Involved**: The Architect, The Inquisitor, The Optimizer, The Skeptic, The Conflict Resolver (mediation), The Conductor (orchestration)

**How It Works**:
1. **Thesis Generation** - The Architect proposes bold solution with rationale (parallel with Optimizer gathering constraints)
2. **Antithesis Construction** - The Inquisitor builds strongest counter-argument, identifying flaws and alternatives
3. **Mediated Conflict** - The Conflict Resolver facilitates structured debate, documenting key tensions
4. **Synthesis Forging** - The Optimizer proposes hybrid solutions preserving strengths while addressing weaknesses
5. **Stress Testing** - The Skeptic attacks synthesis from unexpected angles; Conflict Resolver mediates refinements
6. **Final Integration** - The Conductor synthesizes all perspectives into robust, battle-tested solution

**Example Use Case**: Deciding on system architecture with competing valid approaches (microservices vs monolith, SQL vs NoSQL, custom vs off-the-shelf)

**Why This Flow Matters**: Most AI systems suffer from premature consensus - they converge too quickly on "reasonable" solutions without exploring tension between competing goods. The Dialectic Forge weaponizes disagreement as a feature, using structured conflict to find solutions that genuinely transcend false dichotomies. Valuable for high-stakes decisions with unclear trade-offs.

---

## Summary Statistics

**Total Flows Proposed**: 14
**Pattern Type Distribution**:
- Hybrid: 13 flows
- Sequential with Parallel: 1 flow

**Agent Participation Patterns**:
- Most flows involve 5-6 agents (optimal coordination size)
- Common agent combinations emerge (Archaeologist + Architect, Security + Performance, etc.)
- All flows emphasize synthesis/integration as final step

**Thematic Categories**:
1. **Understanding & Documentation** (4 flows): Archaeological Dig, Architecture X-Ray, Knowledge Archaeology, Competitive Intelligence
2. **Quality & Debt Management** (3 flows): Technical Debt Archaeology, Fortress Protocol, Test-Driven Refactoring
3. **Design & Planning** (3 flows): User Story Pipeline, Contract-First Integration, Recursive Complexity Breakdown
4. **Optimization & Improvement** (2 flows): Performance Cascade, Semantic Harmonization
5. **Synthesis & Decision-Making** (2 flows): Cross-Pollination Synthesis, Dialectic Forge

**Next Steps**:
- Document each flow as standalone markdown file in `.claude/flows/`
- Create flow selection guide (when to use which flow)
- Test 2-3 flows on real-world tasks to validate and refine
