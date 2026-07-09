# AV OPERATING SYSTEM
## THE BEST IN THE WORLD

I developed this OS using Rust and following this tutorial:

https://os.phil-opp.com/

I followed the entire thing, but I skipped allocators because I liked the default linked list allocator. Maybe I'll go back and try them out later, but this is good enough for now.

When I started, I had zero Rust knowledge! I went on W3Schools and played around with loops, pointers, and functions before starting this. The complexity completely blew me out of the water.

I understood like the first half, with VGA Buffer and Interrupts. And then... paging is like pure nonsense. I'll go back and review. The blog ends on Async/Await, and leaves much to be desired.

So, I decided to start coding my CLI. I removed the Timer printing "." over and over. It still ticks in the background. 

I created the backspace function and a couple basic commands. It's very limited functionality. But, I will continue working on it, maybe adding a couple games!

Chaos mode was something I created entirely by myself. I have a good amount of Java knowledge, so this wasn't a crazy leap. I updated the enum to have a "next_color" function. Then, I toggled a boolean to indicate if Chaos Mode was on or off. This is unsafe!!! I'll figure out optimal safeness later. Then, I used an if-statement to check Chaos Mode, and switch the color if it was.

Overall, I learned a lot during this project... 25 hours of gibberish. At least it wasn't in C!

## HOW TO USE

I would run this on QEMU.

1. Install from these guys: https://www.qemu.org/
2. Download the latest release (or build it yourself)
3. Open up terminal
4. Run command: 
> `qemu-system-x86_64 -drive file=bootimage-av_os.bin,format=raw`
5. Enjoy the best OS in the world!