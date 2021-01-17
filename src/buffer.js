const fs = require('fs')
const path = require('path')

const vec = require('./vector')

const empty = () => ({
  lines: [''],
  cursor: vec.zero(),
  yScroll: 0,
  filepath: null,
})

const fromFile = (filename) => {
  const filepath = path.resolve(process.cwd(), filename)
  if (!fs.existsSync(filepath)) {
    fs.writeFileSync(filepath, '')
  }

  const lines = fs.readFileSync(filepath).toString().split('\n')

  return {
    ...empty(),
    lines,
    filepath,
  }
}

const currentLine = (buffer) => buffer.lines[buffer.cursor.y]

const move = (d, rows, buffer) => {
  const nextY = Math.min(
    buffer.lines.length - 1,
    Math.max(0, buffer.cursor.y + d.y)
  )
  let yScroll = buffer.yScroll
  const overflow = nextY - yScroll - rows
  if (overflow > 0) {
    yScroll += overflow
  }

  if (yScroll > nextY) {
    yScroll = nextY
  }

  const lineLength = buffer.lines[nextY]?.length ?? 0

  return {
    ...buffer,
    cursor: {
      x: Math.min(lineLength, Math.max(0, buffer.cursor.x + d.x)),
      y: nextY,
    },
    yScroll,
  }
}

const scrollScreen = (dy, rows, buffer) => {
  const prevDist = buffer.cursor.y - buffer.yScroll
  const maxScroll = Math.max(0, buffer.lines.length - rows)
  const nextYScroll = Math.max(0, Math.min(buffer.yScroll + dy, maxScroll))
  const nextY = nextYScroll + prevDist

  return {
    ...buffer,
    cursor: vec.setY(buffer.cursor, nextY),
    yScroll: nextYScroll,
  }
}

const linesToRender = (rows, buffer) =>
  buffer.lines.slice(buffer.yScroll, buffer.yScroll + rows)

const screenCursor = (buffer) => vec.subY(buffer.cursor, buffer.yScroll)

const removeChar = (buffer) => {
  const chars = currentLine(buffer).split('')
  const lines = buffer.lines.slice(0)
  lines[buffer.cursor.y] = [
    ...chars.slice(0, buffer.cursor.x),
    ...chars.slice(buffer.cursor.x + 1),
  ].join('')

  return {
    ...buffer,
    lines,
  }
}

const insertStr = (str, buffer) => {
  const chars = (currentLine(buffer) ?? '').split('')

  const lines = buffer.lines.slice(0)
  lines[buffer.y] = [
    ...chars.slice(0, buffer.cursor.x),
    str,
    ...chars.slice(buffer.cursor.x),
  ].join('')

  return {
    ...buffer,
    lines,
  }
}

const insertLine = (above, buffer) => {
  const y = buffer.cursor.y + (above ? 0 : 1)
  return {
    ...buffer,
    lines: [...buffer.lines.slice(0, y), '', ...buffer.lines.slice(y)],
  }
}

const splitLine = (buffer) => {
  const { x, y } = buffer.cursor
  let previousLine = buffer.lines[y]
  const newLine = previousLine.slice(x)
  previousLine = previousLine.slice(0, x)

  return {
    ...buffer,
    lines: [
      ...buffer.lines.slice(0, y),
      previousLine,
      newLine,
      ...buffer.lines.slice(y + 1),
    ],
  }
}

const joinLine = (rows, buffer) => {
  const index = buffer.cursor.y
  const lineA = currentLine(buffer)
  const lineB = buffer.lines[index + 1]
  if (lineA === undefined || lineB === undefined) {
    return buffer
  }

  if (lineA !== undefined && lineB !== undefined) {
    let newLine = lineA

    if (newLine !== '' && newLine[newLine.length - 1] !== ' ') {
      newLine += ' '
    }

    newLine += lineB

    let dx = 0
    if (lineA.length > 0) {
      dx = lineA.length - buffer.cursor.x
    }

    return move(dx, 0, rows, {
      ...buffer,
      lines: [
        ...buffer.lines.slice(0, index),
        newLine,
        ...buffer.lines.slice(index + 2),
      ],
    })
  }
}

const save = (buffer, filename) => {
  filepath = filename ?? buffer.filepath
  if (!filepath) {
    // TODO: Display error message
    return buffer
  }

  filepath = path.resolve(process.cwd(), filepath)
  const content = buffer.lines.join('\n')
  fs.writeFileSync(filepath, content)

  return {
    ...buffer,
    filepath,
  }
}

const search = (text, buffer) => {
  const currentLine = buffer.lines[buffer.y]
  const xPosition = currentLine.slice(buffer.cursor.x + 1).indexOf(text)
  if (xPosition >= 0) {
    return { x: buffer.cursor.x + 1 + xPosition, y: buffer.cursor.y }
  }

  const nextLines = buffer.lines.slice(buffer.cursor.y + 1)
  for (let i = 0; i < nextLines.length; i++) {
    const line = nextLines[i]
    const xPosition = line.indexOf(text)
    if (xPosition >= 0) {
      return { x: xPosition, y: buffer.cursor.y + 1 + i }
    }
  }

  const previousLines = buffer.lines.slice(0, buffer.cursor.y)
  for (let i = 0; i < previousLines.length; i++) {
    const line = previousLines[i]
    const xPosition = line.indexOf(text)
    if (xPosition >= 0) {
      return { x: xPosition, y: i }
    }
  }

  return null
}

module.exports = {
  empty,
  fromFile,
  move,
  scrollScreen,
  linesToRender,
  screenCursor,
  removeChar,
  insertStr,
  insertLine,
  splitLine,
  joinLine,
  save,
  search,
}
