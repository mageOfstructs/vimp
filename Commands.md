# Commands Reference

## Coords

- all coordinates are interpreted as a percentage of the canvas
  (which should take up your entire window size)
- absolute: ``a<x>;<y>``
- relative: ``<x>(h|j|k|l);<y>(h|j|k|l)``
  - Note: negative numbers aren't supported (yet)

## Move Cursor

- ``<coords>``
- Note: If you want to move your cursor with one relative motion,
  press enter after you entered it (don't input the semicolon)

## Forms

- Line: ``l<coords_of_endpoint>``
  - starting point will be cursor pos
  
- Rectangle: ``r<coords_of_endpoint>``
  - starting point will be cursor pos
  
- Text: ``t<coords_of_starting_point>``
  - You will be prompted to enter a text
- Circle: ``c[coords_of_middle_point]<radius>``

## Undo

- Backspace works as expected
- ``u``: undo last command executed
- ``U``: redo last command, which has been undone
  - Press escape to clear command buffer (delete current command)

## Color

- Default is red/black
- Suffix command with ``@<color>`` to override default
- all ways of specifying colors in html are allowed
  - Note: Yes, this is a potential security risk

## Select Mode

- similar to how clicking links works in vim browser extensions
- Enter Select Mode: ``e``
- select forms: ``a,b,c,...<CR>``
- To delete: ``d``
- To copy: ``y``
  - To paste later: ``p``
  - Forms will be placed relative to the cursor, so make sure to move it before pasting

## Fast Coord System

- This is probably the first unique feature of vimp
- Syntax: ``<direction><distance>``
- The coords get calculated by moving ``distance`` units (percentage of canvas ofc)
  in ``direction`` with the cursor as starting point.
- ``direction`` can be one of 8 in a "star" system

```text
q w e
a ✴ d
y x c
```

- the ``distance`` system is inspired by roman numerals
  - q is 5 units, e is 15, r is 25, t is 50 and z is 75
  - multiple letters will be summed (no weird subtraction rules here)

- TODO: rewrite docs for fcs (will never get done)
