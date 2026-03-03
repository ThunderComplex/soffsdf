# SoffSDF  

SDF renderer implemented as a pure software renderer.  
Note that this code is Windows only. I am grabbing the raw HWND and use GDI to put pixels on the screen.  
Funnily enough, this is significantly harder than using OpenGL or Vulkan and the code looks horrible. On the flipside, the renderer is basically just creating raw, simple bitmaps, making it much more portable.  

## Architecture  

**winit**: Handles the windowing and event loop  
**windows**: Creates a GDI Bitmap for the window and handles blitting image data to the screen  
**renderer**: The meat of the program. A wholly platform independent software renderer. 
