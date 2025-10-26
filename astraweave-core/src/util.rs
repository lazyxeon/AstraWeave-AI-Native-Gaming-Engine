use crate::IVec2;

pub fn manhattan(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_same_position() {
        let a = IVec2 { x: 5, y: 10 };
        let b = IVec2 { x: 5, y: 10 };
        assert_eq!(manhattan(a, b), 0);
    }

    #[test]
    fn test_manhattan_horizontal() {
        let a = IVec2 { x: 0, y: 0 };
        let b = IVec2 { x: 5, y: 0 };
        assert_eq!(manhattan(a, b), 5);
    }

    #[test]
    fn test_manhattan_vertical() {
        let a = IVec2 { x: 0, y: 0 };
        let b = IVec2 { x: 0, y: 7 };
        assert_eq!(manhattan(a, b), 7);
    }

    #[test]
    fn test_manhattan_diagonal() {
        let a = IVec2 { x: 0, y: 0 };
        let b = IVec2 { x: 3, y: 4 };
        assert_eq!(manhattan(a, b), 7); // 3 + 4 = 7
    }

    #[test]
    fn test_manhattan_negative_coords() {
        let a = IVec2 { x: -5, y: -10 };
        let b = IVec2 { x: 5, y: 10 };
        assert_eq!(manhattan(a, b), 30); // |5-(-5)| + |10-(-10)| = 10 + 20 = 30
    }

    #[test]
    fn test_manhattan_mixed_coords() {
        let a = IVec2 { x: -3, y: 5 };
        let b = IVec2 { x: 2, y: -1 };
        assert_eq!(manhattan(a, b), 11); // |2-(-3)| + |-1-5| = 5 + 6 = 11
    }

    #[test]
    fn test_manhattan_symmetric() {
        let a = IVec2 { x: 10, y: 20 };
        let b = IVec2 { x: 5, y: 15 };
        assert_eq!(manhattan(a, b), manhattan(b, a)); // Distance should be symmetric
    }
}
