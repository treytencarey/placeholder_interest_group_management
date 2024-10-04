# Interest management + Group management

An example that shows using Lightyear to perform interest management + group management.
Currently when both are used, replication breaks when moving rooms.

## Trying out the bugs

- Run a server with `cargo run -- server`
- Run client 1 with `cargo run -- client -c 1`
- Run client 2 with `cargo run -- client -c 2`

### Bug 1
Wait 5 seconds, a timer completes that's supposed to update the player text. When not using group management (PlayerText is directly on the PlayerBundle instead of a PlayerTextBundle), the text is correctly updated.

### Bug 2
Move one of the clients around. After they switch rooms, try to move them back to the same room. Notice it never works again.