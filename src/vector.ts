export interface Vec {
  x: number
  y: number
}

export const zero = (): Vec => ({
  x: 0,
  y: 0,
})

export const one = (): Vec => ({
  x: 1,
  y: 1,
})

export const setX = (vec: Vec, x: number): Vec => ({
  x,
  y: vec.y,
})

export const setY = (vec: Vec, y: number): Vec => ({
  x: vec.x,
  y,
})

export const add = (v1: Vec, v2: Vec): Vec => ({
  x: v1.x + v2.x,
  y: v1.y + v2.y,
})

export const addY = (vec: Vec, y: number): Vec => ({
  ...vec,
  y: vec.y + y,
})

export const subY = (vec: Vec, y: number): Vec => ({
  ...vec,
  y: vec.y - y,
})

export const sub = (v1: Vec, v2: Vec): Vec => ({
  x: v1.x - v2.x,
  y: v1.y - v2.y,
})

export const mul = (v1: Vec, v2: Vec): Vec => ({
  x: v1.x * v2.x,
  y: v1.y * v2.y,
})

export const div = (v1: Vec, v2: Vec): Vec => ({
  x: v1.x / v2.y,
  y: v1.y / v2.y,
})

export const max = (v1: Vec, v2: Vec): Vec => ({
  x: Math.max(v1.x, v2.x),
  y: Math.max(v1.y, v2.y),
})

export const min = (v1: Vec, v2: Vec) => ({
  x: Math.min(v1.x, v2.x),
  y: Math.min(v1.y, v2.y),
})
