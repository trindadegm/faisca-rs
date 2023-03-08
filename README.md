# [WIP] Faisca Renderer
This is a side project of mine. It was created to learn Vulkan. I went outside
of my zone of confort when dealing with synchronization on this project. I was
thinking about doing things less in a less synchronous manner. So, because I
really wasn't basing in anything specific, I ended up with a very weird
application design.

I decided to just go with it because it was fun doing things in a different way.

This isn't the original faisca renderer that I made. I have a gitlab repository
with an OpenGL 2D renderer that I also called faisca. The renderer itself in
that case was made in C++, that is the reason I'm calling the library here
`faisca-rs`. I'm writting it in Rust this time! The reason is simply because C++
with Android was giving me more trouble than Rust with Android. So I decided to
decrease the C++ side a little more and see how that goes.

The initial reason for initializing the application from C++ rather than Rust
has to do with SDL initialization in Android, which also expects you to load your
application (C++ or Rust) via a shared object (a library).