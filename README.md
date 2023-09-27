# Not Oblivion XML

The Elder Scrolls IV: Oblivion has an absolutely insane format for menu panels. It is [conveniently documented by the CS wiki](https://cs.elderscrolls.com/index.php?title=Oblivion_XML_Reference), but remains obtuse and difficult for humans to parse. The aim of this project is to supply a format which is both easy to read and easy to transpile to Oblivion-compatible XML.

In other words, write this:

```text
rect name="container":
    locus: &true;
    x: 0
    y: 0
    width: 500
    // Enforce a 16:9 aspect ratio
    height: me().width / 16 * 9

    image name="icon":
        filename: some_picture\.dds
        // Set the image to use its "natural" size:
        width: me().filewidth
        height: me().fileheight
        // Position this image on the bottom-right corner of its container.
        x: parent().width - me().width
        y: parent().height - me().height
```

and it turns into this:

```xml
<rect name="container">
   <locus>&true;</locus>
   <x>0</x>
   <y>0</y>
   <width>500</width>
   <height> <!-- Enforce a 16:9 aspect ratio -->
      <copy src="me()" trait="width" />
      <div>16</div>
      <mult>9</mult>
      <!--
         The value is computed at run-time. This code means: 
         copy my width, divide it by 16, and multiply it by 9.
      -->
   </height>

   <image name="icon">
      <filename>some_picture.dds</filename>
      <!--
         Set the image to use its "natural" size:
      -->
      <width> <copy src="me()" trait="filewidth" /></width>
      <height><copy src="me()" trait="fileheight" /></height>
      <!--
         Position this image on the bottom-right corner of its container.
      -->
      <x>
         <copy src="parent()" trait="width" />
         <sub  src="me()"     trait="width" />
      </x>
      <y>
         <copy src="parent()" trait="height" />
         <sub  src="me()"     trait="height" />
      </y>
   </image>
</rect>
```

Snippet sampled from CS wiki. Note, though, that comments are not at present preserved.
