pub trait WrapScalar<T> {
    fn wrap(&self, a: T, b: T) -> T;
}

impl WrapScalar<f32> for f32 {
    #[inline]
    fn wrap(&self, a: f32, b: f32) -> f32 {
        if *self < a { b }
        else if *self > b { a }
        else { *self }
    }
}

pub trait BetweenScalar<T> {
    fn between(&self, a: T, b: T) -> bool;
}

impl BetweenScalar<f32> for f32 {
    #[inline]
    fn between(&self, a: f32, b: f32) -> bool {
        a <= *self && *self <= b
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub fn vector_2f(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

#[inline]
pub fn vector_2f_zero() -> Vector2f {
    Vector2f { x: 0.0, y: 0.0 }
}

#[inline]
pub fn vector_2f_unitx() -> Vector2f {
    Vector2f { x: 1.0, y: 0.0 }
}

#[inline]
pub fn vector_2f_unity() -> Vector2f {
    Vector2f { x: 0.0, y: 1.0 }
}

#[inline]
pub fn vector_2f_normalize(v: Vector2f) -> Vector2f {
    let len = vector_2f_length(v);
    let result = Vector2f {
        x: v.x/len,
        y: v.y/len,
    };
    result
}

#[inline]
pub fn vector_2f_length(v: Vector2f) -> f32 {
    (v.x*v.x + v.y*v.y).sqrt()
}


#[inline]
pub fn vector_2f_length_squard(v: Vector2f) -> f32 {
    v.x*v.x + v.y*v.y
}

#[inline]
pub fn vector_2f_scale(v: Vector2f, s: f32) -> Vector2f {
    let result = Vector2f { 
        x: v.x*s,
        y: v.y*s
    };
    result
}

#[inline]
pub fn vector_2f_add(a: Vector2f, b: Vector2f) -> Vector2f {
    Vector2f {
        x: a.x + b.x, 
        y: a.y + b.y, 
    }
}

#[inline]
pub fn vector_2f_sub(a: Vector2f, b: Vector2f) -> Vector2f {
    Vector2f {
        x: a.x - b.x, 
        y: a.y - b.y, 
    }
}

#[inline]
pub fn vector_2f_dot() {
    todo!()
}

#[inline]
pub fn vector_2f_cross() {
    todo!()
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn vector_3f(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub fn vector_4f(x: f32, y: f32, z: f32, w: f32) -> Vector4f {
    Vector4f { x, y, z, w }
}





