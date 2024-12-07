Lantern Rust SDK
================

This crate provides all the structures required to interact with Lantern homeservers.

## Models

These are the raw struct definitions for objects.

## API

Contains structures and definitions for the REST API, abstracting into "Commands"

Commands are used within the `Driver` layer to perform actions.

## Driver

A low-level REST interface using Commands to accomplish tasks.

## Client

A thread-safe mid-level abstraction around the API.

## Gateway

WebSocket Gateway object

## Framework

Ready-made bot framework

# License

This project is licensed under GPLv3, but certain parts of the code may be licensed under different licenses.
Please check the license headers in the files for more information. Notably, embed handling is licensed
under MIT/Apache-2.0 for use in the [embed-service](https://github.com/Lantern-chat/embed-service) project.