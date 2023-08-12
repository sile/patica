dotedit
=======

- auto-revert-tail-mode

Rerequisities
-------------

- Full color support by terminal emulator
- Font 

Known issues
------------

- Cannot get key input such as "Ctrl+ " as `termion` dosen't recognize that

Tips
----

### Undo / Redo

Remove or comment out the tail lines using a text editor:
```
$ emacs ...
```

### Include other image

Just append the target file to the editing one:

```console
$ cut ... >> ...
```
