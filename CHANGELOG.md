## Unreleased

- Make labels not required by default
- Don't include `"hash": null` when hashing labels
- Add labels to events in `generator-file` embedded step
- Include filename in error message when generator-file step can't open the file

## v0.4.0 (2022-08-11)

- Add `sink_all_events` option to force recording of all events

## v0.3.0 (2022-08-09)

- Add a linebreak to the output of `sr version`
- Add `print-config` command
- Add the version to the `help` command output
- Improve error messages when an empty value is supplied for a filename or review name
- Pass a host:port pair for `SR_OUTPUT` and `SR_INPUT` instead of just a port
- Don't write "control" events to sink

## v0.2.0 (2022-08-05)
- Add `sr version` command
- Add label embedded step for simple CLI labelling
- Add support for remote URLs as dbs
- Use sockets for communication between steps instead of fifos
- Add Windows support

## v0.1.0 (2022-07-28)
- First release
