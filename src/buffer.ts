import fs from 'fs'
import path from 'path'

import {Vec} from './vector'
import * as vec from './vector'

interface Buff {
  lines: string[]
  cursor: vec.Vec
  yScroll: number
  filepath: string | null
}

export const empty = (): Buff => ({
  lines: [''],
  cursor: vec.zero(),
  yScroll: 0,
  filepath: null,
})

export const fromFile = (filename: string): Buff => {
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

export const currentLine = (buffer: Buff): string | null => buffer.lines[buffer.cursor.y] ?? null

export const move = (d: Vec, rows: number, buffer: Buff): Buff => {
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

export const scrollScreen = (dy: number, rows: number, buffer: Buff): Buff => {
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

export const linesToRender = (rows: number, buffer: Buff): string[] =>
  buffer.lines.slice(buffer.yScroll, buffer.yScroll + rows)

export const screenCursor = (buffer: Buff): Vec => vec.subY(buffer.cursor, buffer.yScroll)

export const removeChar = (buffer: Buff): Buff => {
  const chars = currentLine(buffer)?.split('') ?? []
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

export const insertStr = (str: string, buffer: Buff): Buff => {
  const chars = (currentLine(buffer) ?? '').split('')

  const lines = buffer.lines.slice(0)
  lines[buffer.cursor.y] = [
    ...chars.slice(0, buffer.cursor.x),
    str,
    ...chars.slice(buffer.cursor.x),
  ].join('')

  return {
    ...buffer,
    lines,
  }
}

export const insertLine = (above: boolean, buffer: Buff): Buff => {
  const y = buffer.cursor.y + (above ? 0 : 1)

  return {
    ...buffer,
    lines: [...buffer.lines.slice(0, y), '', ...buffer.lines.slice(y)],
  }
}

export const splitLine = (buffer: Buff): Buff => {
  const { x, y } = buffer.cursor
  let previousLine = buffer.lines[y] ?? ''
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

export const joinLine = (rows: number, buffer: Buff): Buff => {
  const index = buffer.cursor.y
  const lineA = currentLine(buffer)
  const lineB = buffer.lines[index + 1]
  if (lineA === null || lineB === undefined) {
    return buffer
  }

  let newLine: string = lineA

  if (newLine !== '' && newLine[newLine.length - 1] !== ' ') {
    newLine += ' '
  }

  newLine += lineB

  let dx = 0
  if (lineA.length > 0) {
    dx = lineA.length - buffer.cursor.x
  }

  return move({ x: dx, y: 0 }, rows, {
    ...buffer,
    lines: [
      ...buffer.lines.slice(0, index),
      newLine,
      ...buffer.lines.slice(index + 2),
    ],
  })
  
}

export const save = (buffer: Buff, filename?: string) => {
  let filepath = filename ?? buffer.filepath
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

export const search = (text: string, buffer: Buff): Vec | null => {
  const line = currentLine(buffer)
  if (!line) {
    return null
  }

  const xPosition = line.slice(buffer.cursor.x + 1).indexOf(text)
  if (xPosition >= 0) {
    return { x: buffer.cursor.x + 1 + xPosition, y: buffer.cursor.y }
  }

  const nextLines = buffer.lines.slice(buffer.cursor.y + 1)
  for (const [i, line] of nextLines.entries()) {
    const xPosition = line.indexOf(text)
    if (xPosition >= 0) {
      return { x: xPosition, y: buffer.cursor.y + 1 + i }
    }
  }

  const previousLines = buffer.lines.slice(0, buffer.cursor.y)
  for (const [i, line] of previousLines.entries()) {
    const xPosition = line.indexOf(text)
    if (xPosition >= 0) {
      return { x: xPosition, y: i }
    }
  }

  return null
}

const NEXT_WORD_REG = /\s\S/
export const nextWord = (buffer: Buff): Vec | null => {
  const line = currentLine(buffer)
  if (line === null) {
    return null
  }

  const index = line.slice(buffer.cursor.x).match(NEXT_WORD_REG)?.index
  if (index !== undefined) {
    return vec.add(buffer.cursor, { x: index + 1, y: 0 })
  }

  const nextLines = buffer.lines.slice(buffer.cursor.y + 1)
  for (const [i, line] of nextLines.entries()) {
    if (line === '' || line[0] !== ' ') {
      return { x: 0, y: buffer.cursor.y + 1 + i }
    }

    const index = line.match(NEXT_WORD_REG)?.index
    if (index !== undefined) {
      return { x: index + 1, y: buffer.cursor.y + i + 1 }
    }
  }

  return null
}

export const charAt = (vec: Vec, buffer: Buff): string | null => buffer.lines[vec.y]?.[vec.x]??null

export const previousWord = (buffer: Buff): Vec => {
  let start = buffer.cursor.x
  let y = buffer.cursor.y

  if (start === 0) {
    y = Math.max(0, y - 1)
    while (buffer.lines[y]?.length === 0 && y > 0) {
      y -= 1
    }

    return {
      x: (buffer.lines[y]?.length??0) - 1,
      y,
    }
  }

  let searchText = buffer.lines[y]?.slice(0, start)?.trimEnd() ?? ''

  let position = -1
  let match = null
  const re = /\s/g
  while ((match = re.exec(searchText)) !== null) {
    position = match.index
  }

  return {
    x: position + 1,
    y,
  }
}
