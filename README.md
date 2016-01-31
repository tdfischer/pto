# Perpetually Talking Online

[![Build Status](https://travis-ci.org/tdfischer/pto.svg?branch=master)](https://travis-ci.org/tdfischer/pto)
[![Stories in Ready](https://badge.waffle.io/tdfischer/pto.png?label=ready&title=Ready)](https://waffle.io/tdfischer/pto)

An IRC to Matrix bridge.

This is different from the existing appservice on matrix.org. This work in
progress allows IRC users to join a matrix community. Its just a readme/vague
requirements document right now but code is being hacked on! Its for the
hackerspace community https://oob.systems/. Come join us in
#pto:oob.systems.

The idea being that one could point irssi at an IRC "daemon" behind a .onion
address, and have access to the channels that are local to a homeserver. Joining
\#the-oob would result in the user's matrix user joining \#the-oob:oob.systems, if
the IRCd is pointed to serve up the oob.systems namespace. Users can use SASL
auth to login with their existing oob.systems matrix account.

It would be impossible to join channels outside the homeserver for simplicity of
implementation, and really, this is about building a tight-knit community. One
should still be able to talk with everyone who is in the channel, including
users from outside the homeserver and handle various channel management tasks
such as kicks, bans, topics, etc.

This provides a *super* low level interface to onboarding new users to an
existing (or new!) Matrix community.

## Building this sucker

You'll need the following ingredients: 

- Rust >= 1.5.0: https://www.rust-lang.org/
- Cargo (any recent)

Once those two are installed, you can build PTO as follows:

  ``$ cargo build``

PTO can then be ran:

  ``$ cargo run https://matrix.org/_matrix/client/api/v1/``

Specify a different URL to use a different matrix server.

Or the appropriate binary named ./target/\*/pto

## Configuration

Currently configuration is limited to modifying hardcoded strings in various
places. Thank you for getting this far though! I would absolutely love a patch
<3

The following are hardcoded defaults:

- Listens on 127.0.0.1:8001
- Requires SSL
- Requires files named ./pto.crt and ./pto.key for a SSL certificate and key, respectively

## Usage

By default, PTO will listen on localhost:8001 for an IRC client to connect with
an appropriate username and password. The username and password supplied through
the IRC connection will be used to login to matrix.

# TODO

Check out the Github issues for the project.
