// Final route test correction.
// CI test update.
// Route planner updated for non-overlapping vertical footprint layers.
// FunctionRS Auto Miner route planner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiningMode { OneByOne, ThreeByOne, ThreeByTwo, ThreeByThree }

impl MiningMode {
    pub const fn footprint(self) -> (i32, i32) {
        match self {
            Self::OneByOne => (1, 1),
            Self::ThreeByOne => (3, 1),
            Self::ThreeByTwo => (3, 2),
            Self::ThreeByThree => (3, 3),
        }
    }
}

impl Default for MiningMode {
    fn default() -> Self { Self::ThreeByOne }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementMode { Walk, Mixed, Sprint, SprintJump }

impl Default for MovementMode {
    fn default() -> Self { Self::Mixed }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds { pub start: BlockPos, pub end: BlockPos }

impl Bounds {
    pub fn normalized(self) -> Self {
        Self {
            start: BlockPos::new(self.start.x.min(self.end.x), self.start.y.min(self.end.y), self.start.z.min(self.end.z)),
            end: BlockPos::new(self.start.x.max(self.end.x), self.start.y.max(self.end.y), self.start.z.max(self.end.z)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteStep { pub pos: BlockPos, pub layer: i32 }

pub fn plan_route(bounds: Bounds, mode: MiningMode) -> Vec<RouteStep> {
    let b = bounds.normalized();
    let (vertical, horizontal) = mode.footprint();
    let mut out = Vec::new();
    // Each route layer is a footprint-height band, so 3-high modes do not overlap.
    let mut y = b.start.y;
    while y <= b.end.y {
        let mut z = b.start.z;
        while z <= b.end.z {
            let mut x = b.start.x;
            while x <= b.end.x {
                for dy in 0..vertical {
                    for dx in 0..horizontal {
                        let px = x + dx;
                        let py = y + dy;
                        if px <= b.end.x && py <= b.end.y {
                            out.push(RouteStep { pos: BlockPos::new(px, py, z), layer: y });
                        }
                    }
                }
                x += horizontal;
            }
            z += 1;
        }
        y += vertical;
    }
    out
}

#[derive(Clone, Debug)]
pub struct AutoMinerState {
    pub enabled: bool,
    pub start: Option<BlockPos>,
    pub end: Option<BlockPos>,
    pub mode: MiningMode,
    pub movement: MovementMode,
    pub route: Vec<RouteStep>,
    pub cursor: usize,
}

impl Default for AutoMinerState {
    fn default() -> Self {
        Self { enabled: false, start: None, end: None, mode: MiningMode::default(), movement: MovementMode::default(), route: Vec::new(), cursor: 0 }
    }
}

impl AutoMinerState {
    pub fn rebuild(&mut self) {
        self.route.clear();
        self.cursor = 0;
        if let (Some(start), Some(end)) = (self.start, self.end) {
            self.route = plan_route(Bounds { start, end }, self.mode);
        }
    }

    pub fn set_start(&mut self, pos: BlockPos) { self.start = Some(pos); self.rebuild(); }
    pub fn set_end(&mut self, pos: BlockPos) { self.end = Some(pos); self.rebuild(); }
    pub fn set_mode(&mut self, mode: MiningMode) { self.mode = mode; self.rebuild(); }
    pub fn next_target(&self) -> Option<BlockPos> { self.route.get(self.cursor).map(|s| s.pos) }
    pub fn advance(&mut self) {
        self.cursor += 1;
        if self.cursor >= self.route.len() { self.enabled = false; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_is_layer_first() {
        let r = plan_route(Bounds { start: BlockPos::new(0, 10, 0), end: BlockPos::new(1, 11, 0) }, MiningMode::OneByOne);
        assert_eq!(r.iter().map(|s| s.pos.y).collect::<Vec<_>>(), vec![10, 10, 11, 11]);
    }
    #[test]
    fn footprint_sizes() {
        assert_eq!(plan_route(Bounds { start: BlockPos::new(0, 0, 0), end: BlockPos::new(2, 2, 0) }, MiningMode::ThreeByThree).len(), 9);
        assert_eq!(plan_route(Bounds { start: BlockPos::new(0, 0, 0), end: BlockPos::new(1, 2, 0) }, MiningMode::ThreeByTwo).len(), 6);
    }
    #[test]
    fn finished_route_disables() {
        let mut s = AutoMinerState::default();
        s.set_start(BlockPos::new(0, 0, 0));
        s.set_end(BlockPos::new(0, 0, 0));
        s.enabled = true;
        s.advance();
        assert!(!s.enabled);
    }
}