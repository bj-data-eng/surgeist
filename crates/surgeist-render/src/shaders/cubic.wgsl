fn mitchell_netravali(distance_value: f32) -> f32 {
    let distance = abs(distance_value);
    let squared = distance * distance;
    let cubed = squared * distance;
    if (distance < 1.0) {
        return (7.0 * cubed - 12.0 * squared + 5.3333335) / 6.0;
    }
    if (distance < 2.0) {
        return (-2.3333335 * cubed + 12.0 * squared - 20.0 * distance + 10.666667) / 6.0;
    }
    return 0.0;
}
