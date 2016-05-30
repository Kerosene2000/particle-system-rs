static float back_ease_out(float t, float b, float c, float d) {
    float s = 1.70158f;
    float inner_t = (t / d) - 1.0f;
    return (c * (inner_t * inner_t * ((s + 1.0f) * inner_t + s) + 1.0f) + b);
}

static float elastic_ease_out(float t, float b, float c, float d) {
    if (t == 0.0f) {
        return b;
    }

    float inner_t = t / d;
    if (inner_t == 1.0f) {
        return b + c;
    }

    float p = d * 0.3f;
    float a = c;
    float s = p / 4.0f;
    float temp = (inner_t * d - s) * (2.0f * M_PI_F) / p;
    return (a * pow(2.0f, -10.0f * inner_t) * sin(temp) + c + b);
}

#define EASING_ANIMATION elastic_ease_out

__kernel void update_animation(global float3 const * const restrict from_vec,
                               global float3 const * const restrict to_vec,
                               global float3 * const restrict positions,
                               float time, float duration) {
    size_t const idx = get_global_id(0);
    float3 const from = from_vec[idx];
    float3 const to = to_vec[idx];

    positions[idx] = (float3)(EASING_ANIMATION(time, from.x, to.x - from.x, duration),
                              EASING_ANIMATION(time, from.y, to.y - from.y, duration),
                              EASING_ANIMATION(time, from.z, to.z - from.z, duration));
}

static size_t  xorshift64star(size_t x) {
    x ^= x >> 12; // a
    x ^= x << 25; // b
    x ^= x >> 27; // c
    return x * size_t(2685821657736338717);
}

__kernel void init_rand_sphere_animation(global float3 const * const restrict positions,
                                         global float3 * const restrict from_vec,
                                         global float3 * const restrict to_vec,
                                         global float3 * const restrict velocities) {
    size_t const idx = get_global_id(0);

    float const scaling = 1.f / 20.f;
    size_t const diameter = 20;
    float const x = (float)(xorshift64star(idx >> 3) % (diameter * 100)) / 100.f;
    float const y = (float)(xorshift64star(idx << 2) % (diameter * 100)) / 100.f;
    float const z = (float)(xorshift64star(idx >> 2) % (diameter * 100)) / 100.f;

    // http://math.stackexchange.com/questions/1176761/point-lies-inside-of-the-sphere
    from_vec[idx] = positions[idx];
    float3 center = (float3)(10.f, 10.f, 10.f);
    to_vec[idx] = (float3)(x, y, z);
    float dist = distance(to_vec[idx], center);
    if (dist > diameter / 2.f) {
        to_vec[idx] = center;
    }
    to_vec[idx] -= center;
    to_vec[idx] *= scaling;
    velocities[idx] = (float3)(0.0f, 0.0f, 0.0f);
}

__kernel void init_rand_cube_animation(global float3 const * const restrict positions,
                                       global float3 * const restrict from_vec,
                                       global float3 * const restrict to_vec,
                                       global float3 * const restrict velocities) {
    size_t const idx = get_global_id(0);

    float const scaling = 1.f / 20.f;
    size_t const diameter = 20;
    float const x = (float)(xorshift64star(idx << 3) % (diameter * 100)) / 100.f;
    float const y = (float)(xorshift64star(idx >> 2) % (diameter * 100)) / 100.f;
    float const z = (float)(xorshift64star(idx << 2) % (diameter * 100)) / 100.f;

    from_vec[idx] = positions[idx];
    float3 center = (float3)(10.f, 10.f, 10.f);
    to_vec[idx] = (float3)(x, y, z);
    float dist = distance(to_vec[idx], center);
    if (dist > diameter / 2.f) {
        to_vec[idx] = to_vec[idx];
    }
    to_vec[idx] -= center;
    to_vec[idx] *= scaling;
    velocities[idx] = (float3)(0.0f, 0.0f, 0.0f);
}

__kernel void init_cube_animation(global float3 const * const restrict positions,
                                  global float3 * const restrict from_vec,
                                  global float3 * const restrict to_vec,
                                  global float3 * const restrict velocities) {
    size_t const idx = get_global_id(0);
    size_t const number_particles = get_global_size(0);
    size_t const side_particles = cbrt((float)number_particles); // FIXME compute this one time
    size_t const particles_left = number_particles - (side_particles * side_particles * side_particles);
    float const spacing = 1.0f / (float)side_particles;
    from_vec[idx] = positions[idx];
    if (idx >= number_particles - particles_left) { // FIXME not on (0, 0, 0)
        to_vec[idx] = (float3)(0.0f, 0.0f, 0.0f);
    }
    else {
        to_vec[idx] = (float3)((idx / (side_particles * side_particles)) * spacing,
                               ((idx / side_particles) % side_particles) * spacing,
                               (idx % side_particles) * spacing);
    }
    to_vec[idx] -= ((float)(side_particles - 1) / 2.0f) * spacing;
    velocities[idx] = (float3)(0.0f, 0.0f, 0.0f);
}

__kernel void update_gravitation(global float3 * const restrict positions,
                                 global float3 * const restrict velocities,
                                 float3 gravity_point,
                                 float t) {
    size_t const idx = get_global_id(0);
    positions[idx] += gravity_point;
}
