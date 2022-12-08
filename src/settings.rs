#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Levels {
    // default
    ONE = 1,
    // perspective
    TWO = 2,
    // depth
    THREE = 3,
    // msaa
    FOUR = 4,
    // lighting
    FIVE = 5,
}
