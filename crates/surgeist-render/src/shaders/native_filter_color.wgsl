// Native policy treats subnormal alpha as effective zero before any division.
const MIN_ALPHA: f32 = 1.1754943508222875e-38;
fn effective(value: vec4<f32>) -> vec4<f32> {
    if value.a < MIN_ALPHA { return vec4<f32>(0.0); }
    return value;
}
fn decode(value: vec3<f32>) -> vec3<f32> {
    let low = value / 12.92;
    let high = pow((value + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, value <= vec3<f32>(0.04045));
}
fn encode(value: vec3<f32>) -> vec3<f32> {
    let low = value * 12.92;
    let high = 1.055 * pow(max(value, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(high, low, value <= vec3<f32>(0.0031308));
}
fn to_working(value: vec4<f32>) -> vec4<f32> {
    let v = effective(value);
    if v.a == 0.0 { return v; }
    if parameters.flags.y != 0u { return vec4<f32>(decode(v.rgb / v.a) * v.a, v.a); }
    return v;
}
fn from_working(value: vec4<f32>) -> vec4<f32> {
    let v = effective(value);
    if v.a == 0.0 { return v; }
    if parameters.flags.y != 0u { return vec4<f32>(encode(v.rgb / v.a) * v.a, v.a); }
    return v;
}
