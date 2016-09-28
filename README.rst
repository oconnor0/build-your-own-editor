Build Your Own Editor
=====================

:Author: Matthew O'Connor

Build Your Own Editor is intended to be a simple set of libraries and tools
for creating editor-like, console-based applications. I was inspired by kilo_
but wanted to learn Rust and write something that worked on both Windows and 
Unix consoles. This proved to be a challenge as every terminal console library
that I found supported only Unix-like terminals. Inspired by Termbox I decided
to write a Termbox-like library that supported both Windows and Unix consoles.

Textbox
-------

The public API of Textbox is provided in lib.rs_. The library is called
Textbox since I kept mistyping Termbox as Textbox. It is presently implemented
as a wrapper around Termbox on Unix and with direct Windows API calls (through
cloned and modified winapi-rs and wio-rs) on Windows.

View
----

``view`` is the sample/reference user of Textbox. This file is written
using it. Eventually, I intend make ``view`` respect its name and remove
its editing capabilities - probably by providing a tool named ``edit``. The
shared code - specifically buffers - will likely be extracted into a library
for reuse. It currently uses a line-oriented data structure to hold the text.
Inserting a character requires copying all characters to the right of the
insert point out one in the backing vector. This hasn't caused a performance
issue yet.

Current status of ``view``:

- implemented cursor up/down/left/right
- implemented home/end
- implemented page up/down
- implemented backspace/delete/enter/tab
- all "regular characters" on my keyboard appear to work
- status bar displays file name, dirty status, column and line location and max
- save files with ``Ctrl-S``
- quit with ``Ctrl-Q`` (no save/dirty check)
- open multiple files with ``view a b c``
- ``Esc`` closes the current file and goes to the next
- all features work on Windows console and Linux shell
- virtually no error checking/handling
- opening `The Majestic Million CSV`_ - a 75 MB CSV - on a year old i7 takes a fraction of a second

.. _kilo: https://github.com/antirez/kilo
.. _lib.rs: https://github.com/oconnor0/build-your-own-editor/blob/master/textbox/src/lib.rs
.. _`The Majestic Million CSV`: http://downloads.majestic.com/majestic_million.csv

