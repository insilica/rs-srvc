## Unreleased

- Add `uri` property (alias `url`) to labels in `sr.yaml`. The label definition will be retrieved from the URI.
- Add `uri`/`url` property to steps and flows in `sr.yaml`.
- Add `base_uri`/`base_url` property to sr.yaml. This can be a URI containing a base configuration for the sr.yaml, which is further extended by the contents of sr.yaml.
- Fix `json_schema_url` not working with arbitrary URLs.
- Add `json_schema_uri` alias for `json_schema_url`.
- Rename `generator-file` embedded step to `generator`, making `generator-file` an alias.
- Support URL arguments to `generator`.
- Clean up temporary directories when a step fails.
- Cancel the other steps when one step fails.

## v0.8.0 (2022-11-07)

- Add a `port` config option for the embedded `html` step.

## v0.7.0 (2022-11-03)

- Show an error message at the start of review when reviewer is not a
  valid URL.
- Add embedded `html` step to serve HTML5 files or URLs.
- Add `json_schema` property to labels in sr.yaml. This allows specifying schemas inline.

## v0.6.0 (2022-08-30)

- Allow forcing a specific timestamp value via `SR_TIMESTAMP_OVERRIDE` env var. This helps with reproducibility and testing.
- Use CRLF line endings for text files on Windows
- Add json_schema_url property to label definitions
- Validate label answers against the label's JSON schema
- Add `hash` command to add hashes to an event stream

## v0.5.0 (2022-08-17)

- Make labels not required by default
- Don't include `"hash": null` when hashing labels
- Add labels to events in `generator-file` embedded step
- Include filename in error message when generator-file step can't open the file
- Pretty-print a step when it fails to start

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
