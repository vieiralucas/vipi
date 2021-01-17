const zero = () => ({
  x: 0,
  y: 0,
})

const one = () => ({
  x: 1,
  y: 1,
})

const setX = (vec, x) => ({
  x,
  y: vec.y,
})

const setY = (vec, y) => ({
  x: vec.x,
  y,
})

const add = (v1, v2) => ({
  x: v1.x + v2.x,
  y: v1.y + v2.y,
})

const addY = (vec, y) => ({
  ...vec,
  y: vec.y + y,
})

const subY = (vec, y) => ({
  ...vec,
  y: vec.y - y,
})

const sub = (v1, v2) => ({
  x: v1.x - v2.x,
  y: v1.y - v2.y,
})

const mul = (v1, v2) => ({
  x: v1.x * v2.x,
  y: v1.y * v2.y,
})

const div = (v1, v2) => ({
  x: v1.x / v2.y,
  y: v1.y / v2.y,
})

const max = (v1, v2) => ({
  x: Math.max(v1.x, v2.x),
  y: Math.max(v1.y, v2.y),
})

const min = (v1, v2) => ({
  x: Math.min(v1.x, v2.x),
  y: Math.min(v1.y, v2.y),
})

module.exports = {
  zero,
  one,
  setX,
  setY,
  add,
  addY,
  subY,
  sub,
  mul,
  div,
  max,
  min,
}
