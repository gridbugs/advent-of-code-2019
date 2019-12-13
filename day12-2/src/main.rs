type Vec3 = [i32; 3];

#[derive(Debug, PartialEq, Eq, Clone)]
struct Body {
    position: Vec3,
    velocity: Vec3,
}

impl Body {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            position: [x, y, z],
            velocity: [0, 0, 0],
        }
    }
}

fn update_velocity_single_body(left: &mut Body, right: &mut Body) {
    for axis in 0..3 {
        if left.position[axis] < right.position[axis] {
            left.velocity[axis] += 1;
            right.velocity[axis] -= 1;
        } else if left.position[axis] > right.position[axis] {
            left.velocity[axis] -= 1;
            right.velocity[axis] += 1;
        }
    }
}

fn apply_gravity(bodies: &mut [Body]) {
    for i in 0..bodies.len() {
        for j in 0..i {
            let (left, right) = bodies.split_at_mut(i);
            let left = &mut left[j];
            let right = &mut right[0];
            update_velocity_single_body(left, right);
        }
    }
}

fn apply_velocity(bodies: &mut [Body]) {
    for body in bodies {
        for axis in 0..3 {
            body.position[axis] += body.velocity[axis];
        }
    }
}

fn bodies_axis(bodies: &[Body], axis: usize) -> Vec<(i32, i32)> {
    bodies
        .iter()
        .map(|b| (b.position[axis], b.velocity[axis]))
        .collect()
}

fn num_steps_for_axis(mut bodies: Vec<Body>, axis: usize) -> usize {
    let original = bodies_axis(&bodies, axis);
    let mut count = 0;
    loop {
        count += 1;
        apply_gravity(&mut bodies);
        apply_velocity(&mut bodies);
        if original == bodies_axis(&bodies, axis) {
            break;
        }
    }
    count
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

fn lcm(a: usize, b: usize) -> usize {
    (a * b) / gcd(a, b)
}

fn main() {
    let bodies = vec![
        Body::new(-7, -1, 6),
        Body::new(6, -9, -9),
        Body::new(-12, 2, -7),
        Body::new(4, -17, -12),
    ];
    let x = num_steps_for_axis(bodies.clone(), 0);
    let y = num_steps_for_axis(bodies.clone(), 1);
    let z = num_steps_for_axis(bodies.clone(), 2);
    println!("{}", lcm(lcm(x, y), z));
}
