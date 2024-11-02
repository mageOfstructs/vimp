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

## Undo

- Backspace works as expected
- ``r``: undo last command executed
- ``R``: redo last command, which has been undone
