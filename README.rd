# Power Take-Off

An IRC to Matrix bridge.

This is different from the existing appservice on matrix.org. This work in
progress allows IRC users to join a matrix community. Its just a readme/vague
requirements document right now but code is being hacked on! Its for the
hackerspace community https://oob.systems/

The idea being that one could point irssi at an IRC "daemon" behind a .onion
address, and have access to the channels that are local to a homeserver. Joining
#the-oob would result in the user's matrix user joining #the-oob:oob.systems, if
the IRCd is pointed to serve up the oob.systems namespace. Users can use SASL
auth to login with their existing oob.systems matrix account.

It would be impossible to join channels outside the homeserver for simplicity of
implementation, and really, this is about building a tight-knit community. One
should still be able to talk with everyone who is in the channel, including
users from outside the homeserver and handle various channel management tasks
such as kicks, bans, topics, etc.

This provides a *super* low level interface to onboarding new users to an
existing (or new!) Matrix community.
