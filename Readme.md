# What is this
This project tries to realize an event- and message-based simulator for generic purposes. It is very similar to the popular [omnet++
project](https://omnetpp.org/).
Obviously this is right now WIP and in no way as developed as omnet++ and if you need something easy to work with / with a nice GUI use that instead. It also has a very active networking framework built on top of it, which does a lot of work for you already (tcp, ethernet, and way more stuff is implemented in it).

At the core it is pretty much the same though, you can send generic messages from modules to other modules that you need to connect beforehand.

# What is missing
1. A better way than printing DOT graphs to visualize the graph
1. The signaling part of omnet++ seems interesting but I am not sure if it is needed to implement all functionality. I feel like it is used but it wouldnt have been necessary
1. Having a NED parser would be fantastic. I would still like to have the possibility to create Modules in rust code but NED files are a convenient way of describing the whole container structure which gets tedious in rust. Maybe there is a way to make it more ergonomic without introducing a secondary language though.  