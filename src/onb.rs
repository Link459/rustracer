use crate::vec3::Vec3;

pub struct ONB {
    axis: [Vec3; 3],
}

impl ONB {
    pub fn new(n: &Vec3) -> Self {
        let mut axis = [Vec3::ZERO; 3];
        axis[2] = n.normalize();
        let a = if axis[2].x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        axis[1] = axis[2].cross(&a).normalize();
        axis[0] = axis[2].cross(&axis[1]);

        return Self { axis };
    }

    pub fn u(&self) -> &Vec3 {
        return &self.axis[0];
    }

    pub fn v(&self) -> &Vec3 {
        return &self.axis[1];
    }

    pub fn w(&self) -> &Vec3 {
        return &self.axis[2];
    }

    pub fn transform(&self, v: &Vec3) -> Vec3 {
        return (v.x * self.axis[0]) + (v.y * self.axis[1]) + (v.z * self.axis[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;
    use super::ONB;

    #[test]
    fn access() {
        let n = Vec3::new(0.6, 0.1, 21.9);
        let _onb = ONB::new(&n);
    }
}
