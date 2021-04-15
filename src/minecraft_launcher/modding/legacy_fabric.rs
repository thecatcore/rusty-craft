const MC_VERSIONS: [&str; 4] = [
    "1.12.2",
    "1.8.9",
    "1.7.10",
    "1.6.4"
];
const LOADER_VERSIONS: &str = "https://maven.legacyfabric.net/net/fabricmc/fabric-loader-1.8.9/maven-metadata.xml";
const PROFILE_NAMING: &str = "fabric-loader-1.8.9-{loader_version}-{mc_version}";