# bsp-pathfinding

Provides a path finding search space generation and A* search algorithm
using Binary Spatial Partitioning.

The navigable space is defined by polygons delimiting the space. A graph of
navigable nodes and edges will be generated.

The algorithm does not impose any constraints on size or angle of the scene
as is the case for grid or quadtree based approaches.

## Usage
```rust
use bsp_pathfinding::*;
use glam::*;
// Define a simple scene
let square = Shape::rect(Vec2::new(50.0, 50.0), Vec2::new(0.0, 0.0));
let left = Shape::rect(Vec2::new(10.0, 200.0), Vec2::new(-200.0, 10.0));
let right = Shape::rect(Vec2::new(10.0, 200.0), Vec2::new(200.0, 10.0));
let bottom = Shape::rect(Vec2::new(200.0, 10.0), Vec2::new(10.0, -200.0));
let top = Shape::rect(Vec2::new(200.0, 10.0), Vec2::new(10.0, 200.0));

// Create navigational context from the scene
let nav = NavigationContext::new([square, left, right, top, bottom].iter().flatten());

// Find a path
let start = Vec2::new(-100.0, 0.0);
let end = Vec2::new(100.0, 30.0);

let path = nav
    .find_path(start, end, heuristics::euclidiean, SearchInfo::default())
    .expect("Failed to find a path");
```

