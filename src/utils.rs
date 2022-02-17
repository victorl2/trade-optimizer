pub fn map_range( to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + s * (to_range.1 - to_range.0)
}

pub fn percentage_difference(a: f32, b: f32) -> f32 {
    if a == b && b == 0.0 {
        return 0.0;
    }
    let min = a.min(b);
    let max = a.max(b);
    (max - min) / max
}

// create a evenly spaced Vec of points between 0 and number
pub fn split_number_in_points(number :u32, amount_of_points: u32) -> Vec<(u32, u32)> {
    if amount_of_points <= 1 || amount_of_points > number {
        panic!("amount_of_points must be greater than 1 and less than or equal to number");
    }
    let mut points: Vec<(u32,u32)> = Vec::new();
    let mut start = 0;
    let mut end = number / amount_of_points;
    while end < number {
        points.push((start, end));
        start = end;
        end += number / amount_of_points;
        if end > number {
            end = number;
        }
    }
    return points;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_range(){
        assert_eq!(map_range((-1000.0, 1000.0), 1.0), 1000.0);
        assert_eq!(map_range((-1000.0, 1000.0), 0.0), -1000.0);
        assert_eq!(map_range((-1000.0, 1000.0), 0.5), 0.0);

        assert_eq!(map_range((0.0, 100.0), 1.0), 100.0);
        assert_eq!(map_range((0.0, 100.0), 0.0), 0.0);
        assert_eq!(map_range((0.0, 100.0), 0.5), 50.0);

        assert_eq!(map_range((2.0, 200.0), 1.0), 200.0);
        assert_eq!(map_range((2.0, 200.0), 0.0), 2.0);
        assert_eq!(map_range((2.0, 200.0), 0.5), 101.0);
    }

    #[test]
    fn test_percentage_difference() {
        assert_eq!(percentage_difference(1.0, 2.0), 0.5);
        assert_eq!(percentage_difference(2.0, 1.0), 0.5);
        assert_eq!(percentage_difference(100.0, 50.0), 0.5);
        assert_eq!(percentage_difference(2.0, 2.0), 0.0);
        assert_eq!(percentage_difference(0.0, 200.0), 1.0);
        assert_eq!(percentage_difference(0.0, 0.0), 0.0);
    }

}