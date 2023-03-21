## Unreleased

- Fix html embedded step sometimes not properly determining the directory when relative URLs are used.
- Fix label-web step setting string maxLength to 0 when no maxLength property exists
- Show json-schema title property for group labels in label-web
- Use the srvcOrder json-schema property to control the order of columns in group labels
- Support strings not in arrays in group labels
- Read the `SRVC_TOKEN` environment variable to set `Authorization: Bearer` tokens for some HTTP requests. This token is used for requests caused by the `base-uri`, `flow-uri`, `label-uri`, and `step-uri` properties, as well as requests to the db when the db is a URL. It is not used for other requests due to security considerations.
- Show "no more documents" message in label-web instead of blank page
- Improve styling of group labels
- Populate label-web inputs when answers exist
- Don't submit a new answer when the current one matches the most recent answer (in label-web)
- Avoid writing duplicate events in html step submit-label-answers
- Add a Skip button to label-web
- Flush events in html step after processing submit-label-answers
- Add log messages when uploading events to a remote

## v0.15.0 (2023-03-02)

- Add event hash to some error messages when writing an event fails.
- Print errors that occur in the step server handler, such as event hash mismatches.
- Add support for SQLite generators and sinks.
- Add `sr docs` command to open the documentation website.
- Add a `--db` option to `sr flow` to specify the db from the command-line.
- Add a `--dir` option to set the working directory.
- Add `sr pull` command to import events from a file or URL.
- Fix label-web step not seeing json-schemas for labels.

## v0.14.1 (2023-02-14)

- Reduce CPU usage during a flow.

## v0.14.0 (2023-02-10)

- (breaking) Don't lower-case the `type` property on labels. `type` no longer has any special significance other than in the `label` embedded step. Its use should be replaced with `json-schema`.
- Recognize short aliases for the json-schema label property. E.g., a label with `json-schema: boolean` is valid. The `boolean` alias corresponds to a JSON Schema that only allows boolean values.
- Add env_logger to support controlling log output through the RUST_LOG environment variable.
- Proxy requests to the remote server when running a remote HTML step. We use this to load static assets.
- Add version number to the JSON config that steps receive.
- Prefix HTML step routes with /srvc. The old routes are deprecated, but will still work for now.
- Fix a case where the generator step could emit a label-answer before the corresponding label ([#3](https://github.com/insilica/rs-srvc/issues/3)).
- Avoid writing duplicate events in embedded steps.

## v0.13.0 (2023-01-24)

- Add git revision to "sr version" output
- Rename `remove-reviewed` embedded step to `skip-reviewed` for clarity. `remove-reviewed` still works as an alias.
- Rename `review` command to `flow`. `review` still works as an alias.

## v0.12.0 (2023-01-10)

- Rename `run_embedded` step property to `run-embedded`. `run_embedded` is still accepted as an alias in `sr.yaml` files. [#2](https://github.com/insilica/rs-srvc/issues/2)
- Rename all other properties with underscores to use hashes instead. The properties in `sr.yaml` still accept underscored names as aliases.
- Allow omitting `db` from sr.yaml. The default value is now `sink.jsonl`.
- Add `label-web` embedded step. It serves a web server with a basic labelling interface.
- Fix an unintentional conversion of label.question to lower-case.
- Tolerate blank lines in event streams. This fixes a crash with the `generator` embedded step when the source file has extra blank lines.

## v0.11.0 (2023-01-09)

- Change the behavior of the generator embedded step to emit label-answers
  right after the document they refer to, regardless of the original event
  order.
- Update html embedded step to also serve static assets in the same directory
  as the HTML file, as well as subdirectories.
- Fix a hang when running 'run-embedded-step html' with a URL argument.

## v0.10.1 (2022-12-30)

- Enable Nix flakes when running a command via `uses`.

## v0.10.0 (2022-12-30)

- Add `uses` property to steps. This can be a reference to a [Nix flake](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#examples) that will be run to execute the step

## v0.9.0 (2022-12-12)

- Add `uri` property (alias `url`) to labels in `sr.yaml`. The label definition will be retrieved from the URI.
- Add `uri`/`url` property to steps and flows in `sr.yaml`.
- Add `base_uri`/`base_url` property to sr.yaml. This can be a URI containing a base configuration for the sr.yaml, which is further extended by the contents of sr.yaml.
- Fix `json_schema_url` not working with arbitrary URLs.
- Add `json_schema_uri` alias for `json_schema_url`.
- Rename `generator-file` embedded step to `generator`, making `generator-file` an alias.
- Support URL arguments to `generator`.
- Clean up temporary directories when a step fails.
- Cancel the other steps when one step fails.
- Add a more specific error message for the case that an event can't be parsed because it is a blank line.

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
