# Prompt 38 — Invent the Variation Tree

Not a list of samples. A family tree of sound. Breeding, branching, and evolving audio.

---

## 1. Concept

### What It Is

```
The variation tree is a living visualization of how your sounds are related.

  - Every sound you generate is a node
  - Every "generate variant" creates child nodes
  - Every "interpolate" merges two branches
  - Every "mutate" tweaks a single parameter
  - Every "favorite" marks a node for future branching

It is:
  ✓ A history of your exploration
  ✓ A map of the sound space you've discovered
  ✓ A tool for navigating back to previous good ideas
  ✓ A breeding ground for new sounds
  ✓ A portfolio of your sonic decisions

It is NOT:
  ✗ A list of files in a folder
  ✗ A flat grid of samples
  ✗ A linear undo history
  ✗ Just a visualization gimmick
```

### Why It Matters

```
Sound design is exploration. The variation tree makes exploration visible.

  Without tree:
    - "Which variant did I like again?" (scroll through 30 files)
    - "I had a good sound 3 generations back" (can't find it)
    - "These two were related but I can't remember how"
    - _Save As..._ × 50

  With tree:
    - All exploration is automatically preserved
    - Lineage shows relationships
    - Any node is one click away
    - Branching encourages experimentation (no fear of losing good ideas)
    - "Liking a sound" and "keeping it" are decoupled
```

---

## 2. Data Model

### Core Types

```python
from dataclasses import dataclass
from typing import Optional, List
from enum import Enum
from datetime import datetime

class NodeType(Enum):
    GENERATION = 'generation'      # From prompt
    VARIANT = 'variant'            # Branch from existing
    INTERPOLATION = 'interpolation' # Blend of two sounds
    MUTATION = 'mutation'          # Single parameter change
    IMPORT = 'import'              # External audio
    RECORDING = 'recording'        # Live recorded
    EDIT = 'edit'                  # User edited (trim, fade, etc.)
    MERGE = 'merge'                # Merge of two branches
    RANDOM_WALK = 'random_walk'   # Latent space exploration step

class MutationType(Enum):
    PITCH = 'pitch'
    DECAY = 'decay'
    PUNCH = 'punch'
    TONE = 'tone'
    SPACE = 'space'
    TEXTURE = 'texture'
    BODY = 'body'
    ATTACK = 'attack'
    RANDOM = 'random'

@dataclass
class SoundNode:
    """A single sound in the variation tree."""
    id: str                    # UUID
    parent_id: Optional[str]   # Parent node ID (None for root)
    type: NodeType
    created_at: datetime
    
    # Generation metadata
    prompt: Optional[str]
    seed: int
    model_version: str
    params: dict               # Full generation parameters
    
    # Audio
    audio_hash: str            # SHA-256 of audio content
    duration_ms: float
    embedding: List[float]     # Sound DNA embedding (768-d)
    
    # User state
    is_favorite: bool
    rating: Optional[int]      # 1-5 stars
    tags: List[str]
    notes: Optional[str]
    
    # Mutation info
    mutation_type: Optional[MutationType]
    mutation_params: Optional[dict]  # What changed
    
    # Interpolation info
    interpolation_source_id: Optional[str]  # Second parent (for interpolation)
    interpolation_ratio: Optional[float]    # 0.0-1.0 blend ratio
    
    # Tree layout
    x: float                   # Layout position (computed)
    y: float                   # Layout position (computed)
    depth: int                 # Distance from root

@dataclass
class VariationTree:
    """The full tree structure."""
    root_id: str
    nodes: dict[str, SoundNode]  # id → node
    edges: list[tuple[str, str]] # (parent_id, child_id)
    
    # Additional edges for interpolation (DAG, not pure tree)
    interpolation_edges: list[tuple[str, str, float]]  # (id1, id2, ratio)
    
    # Metadata
    created_at: datetime
    updated_at: datetime
    project_id: Optional[str]
    user_id: str
```

### Database Schema

```sql
CREATE TABLE sound_nodes (
    id TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES sound_nodes(id),
    type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    
    -- Generation
    prompt TEXT,
    seed INTEGER NOT NULL,
    model_version TEXT NOT NULL,
    params_json TEXT,             -- JSON blob
    
    -- Audio reference
    audio_hash TEXT NOT NULL,
    duration_ms REAL NOT NULL,
    
    -- User state
    is_favorite INTEGER DEFAULT 0,
    rating INTEGER,
    notes TEXT,
    
    -- Mutation
    mutation_type TEXT,
    mutation_params_json TEXT,
    
    -- Interpolation
    interpolation_source_id TEXT REFERENCES sound_nodes(id),
    interpolation_ratio REAL,
    
    -- Layout (computed by layout engine)
    tree_x REAL,
    tree_y REAL,
    depth INTEGER DEFAULT 0
);

CREATE INDEX idx_nodes_parent ON sound_nodes(parent_id);
CREATE INDEX idx_nodes_favorite ON sound_nodes(is_favorite);
CREATE INDEX idx_nodes_created ON sound_nodes(created_at);

CREATE TABLE node_tags (
    node_id TEXT REFERENCES sound_nodes(id),
    tag TEXT NOT NULL,
    source TEXT DEFAULT 'user',     -- 'user', 'auto', 'model'
    confidence REAL,
    PRIMARY KEY (node_id, tag)
);

CREATE TABLE edges (
    parent_id TEXT NOT NULL,
    child_id TEXT NOT NULL,
    edge_type TEXT DEFAULT 'variant',  -- 'variant', 'interpolation', 'edit'
    metadata_json TEXT,
    PRIMARY KEY (parent_id, child_id)
);
```

### Storage

```
Each project has its own tree:
  ~/cShot/projects/{project_name}/tree.db  (SQLite)

Favorites can be cross-project:
  ~/cShot/favorites.db  (union of all starred nodes from all trees)

Tree data is also stored in:
  - Sound provenance metadata (Prompt 34)
  - Export logs
  - Cloud sync (if enabled)
```

---

## 3. UI Model

### Tree Visualization

```
The variation tree renders as an interactive directed graph.

┌───────────────────────────────────────────────────────┐
│  Variation Tree                          [Fit] [Zoom] │
│                                                       │
│           ┌──────┐                                    │
│           │ Root │  ← Initial generation              │
│           │ Kick │                                      │
│           └──┬───┘                                    │
│              │                                        │
│      ┌───────┼───────────┬───────────┐               │
│      ▼       ▼           ▼           ▼               │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                │
│  │ V1   │ │ V2   │ │ V3 ★ │ │ V4   │  ← Variants    │
│  │snare │ │ kick │ │ kick │ │ perc │    (V3 favorited)│
│  └──┬───┘ └──┬───┘ └──┬───┘ └──────┘                │
│     │        │        │                              │
│     ▼        ▼        ▼                              │
│  ┌──────┐ ┌──────┐ ┌──────┐                         │
│  │ V1a  │ │ V2a  │ │ V3a  │  ← Second generation    │
│  │ kick │ │ kick │ │ kick │                         │
│  └──────┘ └──────┘ └──────┘                         │
│              │        │                              │
│              ▼        ▼                              │
│           ┌──────┐ ┌──────┐                         │
│           │ V2b  │ │ V3b  │  ← Third generation      │
│           │ kick │ │ kick │                         │
│           └──────┘ └──────┘                         │
│                    ╲   ╱                            │
│                     ╲ ╱                     ← Interpolation
│                      ╳                             │
│                     ╱ ╲                            │
│                    ╱   ╲                           │
│                ┌──────┐ ┌──────┐                   │
│                │ I1   │ │ I2   │  ← Interpolated   │
│                │ kick │ │ kick │     children       │
│                └──────┘ └──────┘                   │
│                                                       │
│  ★ Favorited  ◉ Current selection  ○ Unheard         │
│  ● Previewed  ⚪ Generated                           │
└───────────────────────────────────────────────────────┘
```

### Node Appearance

```
Each node is a card showing:

  ┌──────────────┐
  │  █▁▃▇▆▄▂     │  ← Mini waveform (32x16px)
  │  ★ Kick 0.3s │  ← Type + duration
  │  dark punchy │  ← Top tags (2)
  │  ◉           │  ← State indicator
  └──────────────┘

States:
  ◉ Current selection (pulsing glow, primary color)
  ★ Favorited node (star icon, amber accent)
  ● Heard/previewed (solid dot)
  ○ Unheard (empty dot)
  ⚪ Favorite branch (dashed border)
  ⊘ Dead end (no further variants explored)
  ⟳ Has hidden children (collapsed branch indicator)
```

### Interaction

```
Click node:      Select + preview sound
Double-click:    Zoom to node, show in detail panel
Right-click:     Context menu:
                   - Set as root for new branching
                   - Favorite/unfavorite
                   - Generate variants
                   - Mutate (submenu: pitch, decay, punch...)
                   - Export
                   - Show provenance
                   - Delete (prune branch)
                   - Compare (A/B with currently selected)

Drag node:       Reposition (manual layout override)
Ctrl+click:      Add to compare list (multi-select for A/B/C)
Shift+click:     Set as second parent for interpolation
Scroll wheel:    Zoom in/out
Click background: Deselect, pan (drag to move)
```

### Layout Algorithm

```python
class TreeLayoutEngine:
    """
    Compute optimal layout for the variation tree.
    Uses a modified hierarchical layout:
      - Root at top
      - Children spread below parent
      - Subtrees don't overlap
      - Favorites get visual priority (more space)
    """
    
    def layout(self, tree: VariationTree) -> dict[str, tuple[float, float]]:
        """Return {node_id: (x, y)} positions."""
        positions = {}
        
        # 1. Compute depth from root
        depths = self.compute_depths(tree)
        
        # 2. Group by depth
        by_depth = defaultdict(list)
        for node_id, depth in depths.items():
            by_depth[depth].append(node_id)
        
        # 3. Compute x-positions within each depth level
        max_depth = max(depths.values())
        for depth in range(max_depth + 1):
            nodes = by_depth[depth]
            
            # Favorite nodes get more horizontal space
            favorites = [n for n in nodes if tree.nodes[n].is_favorite]
            total_width = len(nodes) + len(favorites) * 0.5  # Favorites 1.5x width
            
            spacing = 1.0 / (total_width + 1)
            x_offset = spacing
            
            for node in nodes:
                width_factor = 1.5 if tree.nodes[node].is_favorite else 1.0
                positions[node] = (x_offset + width_factor * spacing / 2, depth)
                x_offset += width_factor * spacing
        
        # 4. Minimize edge crossings (barycenter heuristic)
        positions = self.minimize_crossings(tree, depths, positions, by_depth)
        
        # 5. Scale to viewport
        positions = self.scale_to_viewport(positions, viewport_width=1200, viewport_height=800)
        
        return positions
```

---

## 4. Audio Lineage System

### How Lineage Works

```
When you create a new sound, the tree records its ancestry:

Root (seed=42)
  └── Variant 1 (seed=43, mutation=pitch)
      └── Variant 1a (seed=100, mutation=decay)
          └── Variant 1a-i (seed=200, interpolation with Variant 3)
  
This means you can always trace back:
  "This sound is a pitch-shifted version of a decay-adjusted version of the original."
```

### Lineage Traversal

```python
class LineageTraversal:
    """Navigate the variation tree to understand sound relationships."""
    
    @staticmethod
    def get_ancestors(node_id: str, tree: VariationTree) -> list[str]:
        """Get all ancestors, from parent to root."""
        ancestors = []
        current = tree.nodes[node_id].parent_id
        while current:
            ancestors.append(current)
            current = tree.nodes[current].parent_id
        return ancestors
    
    @staticmethod
    def get_descendants(node_id: str, tree: VariationTree, max_depth=10) -> list[str]:
        """Get all descendants (children, grandchildren, etc.)."""
        descendants = []
        queue = [(node_id, 0)]
        while queue:
            current, depth = queue.pop(0)
            if depth >= max_depth:
                continue
            children = [n.id for n in tree.nodes.values() if n.parent_id == current]
            for child in children:
                descendants.append(child)
                queue.append((child, depth + 1))
        return descendants
    
    @staticmethod
    def get_path(node_a: str, node_b: str, tree: VariationTree) -> list[str]:
        """Find the path between two nodes through common ancestor."""
        a_ancestors = set(LineageTraversal.get_ancestors(node_a, tree))
        b_ancestors = set(LineageTraversal.get_ancestors(node_b, tree))
        
        # Find common ancestor
        common = None
        for ancestor in [node_a] + list(LineageTraversal.get_ancestors(node_a, tree)):
            if ancestor in b_ancestors or ancestor == node_b:
                common = ancestor
                break
        
        if not common:
            return []
        
        # Build path
        a_to_common = [node_a]
        current = node_a
        while current != common:
            current = tree.nodes[current].parent_id
            a_to_common.append(current)
        
        b_to_common = [node_b]
        current = node_b
        while current != common:
            current = tree.nodes[current].parent_id
            b_to_common.append(current)
        
        return a_to_common + list(reversed(b_to_common[1:]))
    
    @staticmethod
    def get_siblings(node_id: str, tree: VariationTree) -> list[str]:
        """Get other children of the same parent."""
        parent = tree.nodes[node_id].parent_id
        if not parent:
            return []
        return [n.id for n in tree.nodes.values() 
                if n.parent_id == parent and n.id != node_id]
```

### Lineage-Based Features

```
1. "Walk Back" — Move up the tree to previous generations
   - Useful when you've gone too far in one direction
   - One click restores a previous version
   
2. "Show Me the Path" — Highlight the branch from root to current
   - Context for where this sound came from
   
3. "Compare Generations" — A/B current sound with its parent
   - Hear what the mutation actually changed
   
4. "Family Resemblance" — Find sounds genetically similar to current
   - Using Sound DNA similarity weighted by tree distance
   
5. "Convergence Detection" — Notice when different branches 
   produce similar sounds → suggest merging
```

---

## 5. Mutation Controls

### Mutation Panel

```
When you click "Mutate" on a node, the mutation panel opens:

┌─────────────────────────────────────────────────────┐
│  Mutate: Kick_03                                     │
│  ─────────────────────                               │
│                                                       │
│  One-Click Mutations:                                 │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐        │
│  │ Pitch  │ │ Decay  │ │ Punch  │ │ Tone   │        │
│  ├────────┤ ├────────┤ ├────────┤ ├────────┤        │
│  │ Space  │ │Texture │ │ Body   │ │Attack  │        │
│  └────────┘ └────────┘ └────────┘ └────────┘        │
│                                                       │
│  Slider Mutations:                                    │
│  Pitch:     ◉───────○───────○───────○─  (+3 semitones)│
│  Decay:     ○───────○───────◉───────○  (0.7x)        │
│  Punch:     ○───────◉───────○───────○  (1.5x)        │
│  Tone:      ○───○───○───◉───○───○───○  (darker)     │
│                                                       │
│  Advanced:                                             │
│  Mutation strength: [─────────●─────] 0.7             │
│  Preserve type: [✓] (keep it a kick, not morph genre) │
│  Random seed: [⟳]                                     │
│                                                       │
│  [Apply as new variant]  [Preview]  [Cancel]          │
└─────────────────────────────────────────────────────┘
```

### Mutation Algorithms

```python
class MutationEngine:
    """Apply perceptually-meaningful mutations to sounds."""
    
    def mutate_pitch(self, audio, sr, semitones=2):
        """Shift pitch while preserving duration and formant structure."""
        # Use phase vocoder or PSOLA for high-quality pitch shift
        n_steps = semitones
        return pitch_shift(audio, sr, n_steps)
    
    def mutate_decay(self, audio, sr, factor=0.7):
        """Scale the decay envelope."""
        envelope = compute_envelope(np.abs(audio))
        # Find where sound drops below 10% of peak
        peak = np.max(envelope)
        decay_start = np.where(envelope > peak * 0.5)[0][-1]
        
        # Apply exponential scaling from decay point
        if factor < 1.0:  # Shorter decay
            new_length = int(decay_start + (len(audio) - decay_start) * factor)
            audio = audio[:new_length]
            # Fade out
            fade = np.linspace(1, 0, len(audio) - decay_start) ** 2
            audio[decay_start:] *= fade
        else:  # Longer decay
            tail = audio[decay_start:]
            stretched = stretch_audio(tail, factor=factor)
            audio = np.concatenate([audio[:decay_start], stretched])
        
        return audio
    
    def mutate_punch(self, audio, sr, amount=1.5):
        """Enhance or reduce attack transient."""
        transient_len = int(0.01 * sr)  # 10ms
        transient = audio[:transient_len].copy()
        body = audio[transient_len:].copy()
        
        transient *= amount
        
        # Optional: add harmonic content to transient
        if amount > 1.5:
            harmonics = np.sin(2 * np.pi * 3000 * np.arange(transient_len) / sr)
            transient += harmonics * np.max(np.abs(transient)) * 0.1
        
        return np.concatenate([transient, body])
    
    def mutate_tone(self, audio, sr, shift='darker'):
        """Shift spectral centroid for tonal change."""
        spec = compute_spectrum(audio, sr)
        
        if shift == 'darker':
            # Low-pass: reduce high frequencies
            cutoff = 4000
            return apply_lowpass(audio, sr, cutoff)
        elif shift == 'brighter':
            # High-shelf: boost highs
            return apply_high_shelf(audio, sr, 3000, 3)
        else:
            return audio
    
    def mutate_space(self, audio, sr, amount=0.3):
        """Add or reduce reverb/space."""
        if amount > 0:
            # Add reverb
            ir = get_room_ir(decay=amount)
            wet = convolve(audio, ir)[:len(audio)]
            return 0.7 * audio + 0.3 * wet
        else:
            # Reduce existing reverb (not perfect, but reduce tail)
            return audio  # Would need dry/wet separation
    
    def mutate_texture(self, audio, sr, texture_type='saturation'):
        """Apply textural processing."""
        if texture_type == 'saturation':
            return soft_clip(audio, threshold=0.6)
        elif texture_type == 'bitcrush':
            return bitcrush(audio, bits=8, rate=22050)
        elif texture_type == 'noise':
            noise = np.random.randn(len(audio)) * 0.01
            return audio + noise
        elif texture_type == 'granular':
            return granular_scatter(audio, sr, grain_size=50)
        return audio
    
    def mutate_body(self, audio, sr, target_body='sub'):
        """Emphasize different body/frequency regions."""
        if target_body == 'sub':
            return apply_low_shelf(audio, sr, 80, 4)
        elif target_body == 'mid':
            return apply_peak(audio, sr, 250, 3)
        elif target_body == 'thin':
            return apply_high_pass(audio, sr, 100)
        return audio
    
    def mutate_attack(self, audio, sr, attack_ms=2):
        """Adjust attack time."""
        original_attack = int(attack_ms * sr / 1000)
        current_attack = detect_attack_time(audio, sr)
        
        if attack_ms < current_attack:
            # Faster attack: emphasize earlier part of transient
            window = int(min(current_attack, 0.01 * sr))
            envelope = np.linspace(0.3, 1.0, window) ** (current_attack / attack_ms)
            audio[:window] *= envelope
        else:
            # Slower attack: smooth the onset
            window = int(attack_ms * sr / 1000)
            envelope = np.linspace(0.1, 1.0, window) ** 0.5
            audio[:window] *= envelope
        
        return audio
    
    def mutate_random(self, audio, sr, strength=0.5):
        """Apply random combination of mutations."""
        mutations = [
            self.mutate_pitch, self.mutate_decay, self.mutate_punch,
            self.mutate_tone, self.mutate_space, self.mutate_texture,
            self.mutate_body, self.mutate_attack
        ]
        # Pick 1-3 random mutations
        n = np.random.randint(1, 4)
        selected = np.random.choice(mutations, n, replace=False)
        
        for mutation in selected:
            amount = 1.0 + (np.random.random() - 0.5) * strength * 2
            audio = mutation(audio, sr, amount=amount)
        
        return audio
```

---

## 6. Combine Two Sounds

### Interpolation

```
Interpolation creates child sounds that blend two parents.

┌──────┐         ┌──────┐
│ A    │         │ B    │
│ kick │         │ snare│
└──┬───┘         └──┬───┘
   ╲               ╱
    ╲  0.25      ╱
     ┌──────────┐
     │ A×0.75   │ = Mostly A, touch of B
     │ + B×0.25 │
     └──────────┘
         │
    ┌────┼────┬────┬────┐
    ▼    ▼    ▼    ▼    ▼
  0.0  0.25 0.50 0.75 1.0
  (A)  blend      blend (B)
```

```python
def interpolate_sounds(audio_a, audio_b, ratio=0.5, sr=44100):
    """
    Blend two sounds in latent space or waveform domain.
    ratio=0.0 → 100% A, ratio=1.0 → 100% B
    """
    # Method 1: Latent interpolation (preferred)
    embedding_a = encode_to_latent(audio_a)
    embedding_b = encode_to_latent(audio_b)
    blended = embedding_a * (1 - ratio) + embedding_b * ratio
    audio = decode_from_latent(blended)
    
    # Method 2: Waveform cross-fade (fallback)
    # Ensure same length
    max_len = max(len(audio_a), len(audio_b))
    audio_a = np.pad(audio_a, (0, max_len - len(audio_a)))
    audio_b = np.pad(audio_b, (0, max_len - len(audio_b)))
    
    # Cross-fade
    audio = audio_a * (1 - ratio) + audio_b * ratio
    
    # Normalize
    peak = np.max(np.abs(audio))
    if peak > 0:
        audio = audio / peak * 0.95
    
    return audio
```

### Cross-Breeding (Advanced)

```
"Crossover" — take different dimensions from each parent:

  Kick A (punchy, short) + Kick B (subby, long)
  → Child: A's transient + B's body
  
  How it works:
    1. Analyze A and B: separate transient from body
    2. Take A's transient (first 50ms, attack profile)
    3. Take B's body (tail, sub frequencies)
    4. Combine: A's transient → B's body
    5. Result: "punchy transient meets long subby decay"
```

```python
def crossover_sounds(audio_a, audio_b, sr=44100, crossover_point_ms=50):
    """
    Take transient from A, body from B.
    A 'genetic crossover' for audio.
    """
    crossover_samples = int(crossover_point_ms * sr / 1000)
    
    # Envelopes
    envelope_a = compute_envelope(np.abs(audio_a))
    envelope_b = compute_envelope(np.abs(audio_b))
    
    # Transient from A
    transient_a = audio_a[:crossover_samples].copy()
    
    # Body from B
    body_b = audio_b[crossover_samples:].copy()
    
    # Ensure same length for body
    # Time-stretch body to match? Or keep A's length?
    target_length = max(len(audio_a), len(audio_b))
    
    child = np.zeros(target_length)
    child[:crossover_samples] = transient_a
    
    body_len = min(len(body_b), target_length - crossover_samples)
    child[crossover_samples:crossover_samples + body_len] = body_b[:body_len]
    
    # Smooth crossfade at junction
    fade_len = int(0.002 * sr)  # 2ms crossfade
    for i in range(fade_len):
        idx = crossover_samples - fade_len + i
        if 0 <= idx < len(child) and idx < len(audio_a):
            t = i / fade_len
            child[idx] = audio_a[idx] * (1 - t) + child[idx] * t
    
    # Normalize
    peak = np.max(np.abs(child))
    if peak > 0:
        child = child / peak * 0.95
    
    return child
```

---

## 7. Save and Export Behavior

### Auto-Save

```
The tree is auto-saved:
  - After every generation/mutation/interpolation
  - To per-project SQLite database
  - No user action required

What gets saved:
  - All nodes and edges
  - Audio content (content-addressed)
  - User state (favorites, ratings, notes, tags)
  - Tree layout (node positions if manually adjusted)
```

### Export Strategies

```
Single sound export:
  - Exports the selected node's audio
  - Includes provenance metadata linking to tree position
  - Filename: {type}_{generation}_{seed}.wav
  - Example: "kick_gen3_seed424242.wav"

Branch export:
  - Export all sounds in a branch as a sample pack
  - Organized as: /Kick/{variant_name}.wav
  - Includes tree.json for reconstruction

Tree export:
  - Full tree as portable format (.cshottree)
  - All audio, metadata, layout preserved
  - Can be shared with other cShot users
  - Can be re-imported to continue exploration
```

### Pack Builder Integration

```
From variation tree → sample pack:

  1. Select nodes in tree (multi-select)
  2. Click "Create Pack"
  3. Pack Builder opens with selected sounds
  4. Auto-organizes by type (Kick/, Snare/, etc.)
  5. Auto-names based on lineage position
  6. User can reorder, rename, add metadata
  7. Export as folder or .zip
```

---

## 8. Tree Management

### Pruning

```
Delete a branch:
  - Select node → right-click → "Prune branch"
  - Removes the node and all descendants
  - Audio files are kept (referenced by other branches might exist)
  - Can undo (prune is soft-delete until tree is cleaned)

Clean up:
  - "Remove unheard branches" — delete nodes never previewed
  - "Collapse similar" — group near-identical nodes (embedding similarity > 0.95)
  - "Keep favorites only" — remove all non-favorited nodes
```

### Search in Tree

```
Search within the tree:
  - Filter by type: "show only kicks"
  - Filter by tag: "dark"
  - Filter by favorite: ★
  - Filter by date: "last session"
  - Filter by rating: "4+ stars"
  - Text search on prompt text and notes
  
  Search results highlight matching nodes.
  Non-matching nodes dim but remain for context.
```

### Statistics

```
Tree Stats panel:
  Total sounds:     47
  Favorites:         8
  Branches explored: 6
  Max depth:         5 generations
  Interpolations:    3
  
  "You've explored 47 variations of this initial kick."
  "Your favorite branch: V3 → V3a → V3b"
  "Most mutated dimension: decay (12 mutations)"
```

---

## Summary

| Feature | What It Does | Why It Matters |
|---------|-------------|----------------|
| Tree visualization | Shows all sound relationships | Exploration becomes visible |
| Lineage tracking | Records parent-child chain | Always know where a sound came from |
| Mutation controls | One-click sound changes | Fast iteration without losing originals |
| Interpolation | Blend two sounds | Create genuinely new hybrids |
| Crossover | Mix transient/body from different sounds | Sound breeding |
| Auto-save | Nothing is ever lost | Fearless experimentation |
| Pruning | Remove unwanted branches | Keep trees manageable |
| Search/filter | Find sounds in large trees | Scale to hundreds of variants |
| Statistics | See your exploration patterns | Understand your creative process |

The variation tree turns sound design from "make a sound, save it, move on" into "breed a family of sounds and watch them evolve."
