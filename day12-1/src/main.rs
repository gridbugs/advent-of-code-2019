type Vec3 = [i32; 3];

#[derive(Debug, PartialEq, Eq)]
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
    fn energy(&self) -> i32 {
        let potential = self.position.iter().map(|p| p.abs()).sum::<i32>();
        let kinetic = self.velocity.iter().map(|v| v.abs()).sum::<i32>();
        potential * kinetic
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

fn main() {
    let mut bodies = vec![
        Body::new(-7, -1, 6),
        Body::new(6, -9, -9),
        Body::new(-12, 2, -7),
        Body::new(4, -17, -12),
    ];
    for _ in 0..1000 {
        apply_gravity(&mut bodies);
        apply_velocity(&mut bodies);
    }
    let total = bodies.iter().map(|b| b.energy()).sum::<i32>();
    println!("{}", total);
}
