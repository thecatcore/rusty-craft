
const MAVEN: &str = "http://dl.liteloader.com/versions/";

const MAIN_CLASS: &str = "net.minecraft.launchwrapper.Launch";

const VERSIONS: [(&str, bool, [&str; 4]); 16] = [
    ("1.12.2", false, ["net.minecraft:launchwrapper:1.12", "", "http://repo.mumfrey.com/content/repositories/liteloader/org/ow2/asm/asm-all/5.2/asm-all-5.2.jar", ""]),
    ("1.12.1", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.12", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.11.2", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.11", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.10.2", true, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.10", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.9.4", true, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.9", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.8.9", false, ["net.minecraft:launchwrapper:1.12", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.8", true, ["net.minecraft:launchwrapper:1.11", "", "org.ow2.asm:asm-all:5.0.3", ""]),
    ("1.7.10", true, ["net.minecraft:launchwrapper:1.11", "", "org.ow2.asm:asm-all:5.0.3", "com.google.guava:guava:16.0"]),
    ("1.7.2", true, ["net.minecraft:launchwrapper:1.9", "", "org.ow2.asm:asm-all:4.1", ""]),
    ("1.6.4", true, ["net.minecraft:launchwrapper:1.8", "lzma:lzma:0.0.1", "", ""]),
    ("1.6.2", true, ["net.minecraft:launchwrapper:1.3", "lzma:lzma:0.0.1", "", ""]),
    ("1.5.2", true, ["", "", "", ""])
];