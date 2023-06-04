# Intro
In this exercise, you will build and document a system that is capable of
serving lines out of a file to network clients.
Please complete this exercise in Rust.
You may use any reference and any (open-source) software/library you can find
to help you build this system, so long as you document your use. However, you
should not actively collaborate with others.

# Specification

Your system should act as a network server that serves individual lines of an
immutable text file over the network to clients using the following protocol:

## `GET nnnn`
* If nnnn is a valid line number for the given text file, return "OK\r\n" and
then the nnnn-th line of the specified text file.
* If nnnn is not a valid line number for the given text file, return "ERR\r\n".
* The first line of the file is line 1 (not line 0).


## `QUIT`
* Disconnect client

## `SHUTDOWN`
* Shutdown the server


The server should listen for TCP connections on port `10497`.

Your server must support multiple simultaneous clients at a time. The system
should perform well for small and large files. The system should perform well
as the number of GET requests per unit time increases.
You may pre-process the text file in any way that you wish so long as the
server behaves correctly.
Server activity should be observable, both live and after closing the program,
       allowing auditing of activity that happened on the server at any time.
       There should be sufficient data to draw conclusions about connection
       lengths, requests made, and any other typical activity insights.

The text file will have the following properties:
* Each line is terminated with a newline ("\n").
* Any given line will fit into memory.
* The line is valid ASCII (e.g. not Unicode).


# Example

Suppose the given file is:
```
the
quick brown
fox jumps over the
lazy dog
```

Then you could imagine the following transcript with your server:
```
    Client => GET 1
    Server <= OK
    Server <= the
    Client => GET -3
    Server <= ERR
    Client => GET 4
    Server <= OK
    Server <= lazy dog
    Client => QUIT
<<`Server disconnects from client 1 >>
    Client => SHUTDOWN
<<`Server disconnects from ALL clients >>
```

# Execution Environment

You may assume that your system will execute on a machine with the following
specifications:
* 64 bit linux environment (we will test on an 20.04 or newer Ubuntu installation)
* 8 GB of memory
* Two 64-bit cores
* 10Gbps ethernet connection
* a 10 GB of root partition
* a 420 GB drive mounted under /mnt
* a 420 GB drive un-mounted

The entire machine is at your disposal.

# Submission
The top-level directory of your submission (or source-tree) should contain
shell scripts to build and run your system, documentation for your system, and
the source code for the system itself.
* build.sh - A script that can be invoked to build your system. This script may
exit without doing anything if your system does not need to be compiled. You
may invoke another tool such as Maven, Ant, GNU make, etc. with this script.
You may download and install any libraries or other programs you feel are
necessary to help you build your system.
* run.sh - A script that takes a single command-line parameter which is the
name of the file to serve. Ultimately, it should start the server you have
built.

We will probably run these scripts manually, but it would be nice if they
worked without manual intervention.

* README - A text file that answers the following questions:
    - How does your system work? (if not addressed in comments in source)
    - How will your system perform as the number of requests per second increases?
    - How will your system perform with a 1 GB file? a 100 GB file? a 1,000 GB file?
    - What documentation, websites, papers, etc did you consult in doing this assignment?
    - What third-party libraries or other tools does the system use?
    - How long did you spend on this exercise?
