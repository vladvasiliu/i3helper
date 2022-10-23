# i3helper

i3helper is a tool that augments i3 by adding functionality based on events.

Its focus is to be small, fast and configurable.


## Status

The project is in a prototype stage. For now, it only follows focus events and allows to switch to the previous focused window by listening for USR1.


## How it works

i3status listens for events either from i3 or from external sources and sends a command to i3.

Communication with i3 is performed through [i3ipc].


## Building

Development happens with the latest stable Rust and the latest stable i3.


## License

This project is released under the terms of the GNU General Public License, version 3.
See [`COPYING`](COPYING) for the full text of the license.

[i3ipc]: https://i3wm.org/docs/ipc.html "i3ipc"
