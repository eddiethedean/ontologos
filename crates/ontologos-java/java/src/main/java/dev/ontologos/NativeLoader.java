package dev.ontologos;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/** Loads the OntoLogos JNI native library. */
final class NativeLoader {
    private static volatile boolean loaded;

    private NativeLoader() {}

    static void load() {
        if (loaded) {
            return;
        }
        synchronized (NativeLoader.class) {
            if (loaded) {
                return;
            }
            String override = System.getProperty("ontologos.native.path");
            if (override != null && !override.isBlank()) {
                System.load(Path.of(override).toAbsolutePath().toString());
                loaded = true;
                return;
            }
            Path workspace = workspaceLibrary();
            if (workspace != null) {
                System.load(workspace.toAbsolutePath().toString());
                loaded = true;
                return;
            }
            try {
                System.loadLibrary("ontologos_jni");
                loaded = true;
                return;
            } catch (UnsatisfiedLinkError ignored) {
                // Fall back to bundled resource.
            }
            Path extracted = extractBundledLibrary();
            System.load(extracted.toAbsolutePath().toString());
            loaded = true;
        }
    }

    private static Path workspaceLibrary() {
        String os = System.getProperty("os.name", "").toLowerCase();
        String fileName;
        if (os.contains("mac")) {
            fileName = "libontologos_jni.dylib";
        } else if (os.contains("linux")) {
            fileName = "libontologos_jni.so";
        } else if (os.contains("win")) {
            fileName = "ontologos_jni.dll";
        } else {
            return null;
        }
        String[] roots = {
            System.getenv("ONTOLOGOS_REPO_ROOT"),
            "../../..",
            "../..",
            "."
        };
        for (String root : roots) {
            if (root == null || root.isBlank()) {
                continue;
            }
            Path candidate = Path.of(root, "target", "release", fileName).toAbsolutePath().normalize();
            if (Files.exists(candidate)) {
                return candidate;
            }
        }
        return null;
    }

    private static Path extractBundledLibrary() {
        String os = System.getProperty("os.name", "").toLowerCase();
        String resourceName;
        String fileName;
        if (os.contains("mac")) {
            resourceName = "/native/libontologos_jni.dylib";
            fileName = "libontologos_jni.dylib";
        } else if (os.contains("linux")) {
            resourceName = "/native/libontologos_jni.so";
            fileName = "libontologos_jni.so";
        } else if (os.contains("win")) {
            resourceName = "/native/ontologos_jni.dll";
            fileName = "ontologos_jni.dll";
        } else {
            throw new UnsatisfiedLinkError("unsupported OS for OntoLogos JNI: " + os);
        }

        try (InputStream in = NativeLoader.class.getResourceAsStream(resourceName)) {
            if (in != null) {
                Path temp = Files.createTempFile("ontologos_jni-", "-" + fileName);
                temp.toFile().deleteOnExit();
                Files.copy(in, temp, StandardCopyOption.REPLACE_EXISTING);
                return temp;
            }
        } catch (IOException error) {
            throw new UnsatisfiedLinkError("failed to extract bundled OntoLogos native library: " + error);
        }

        throw new UnsatisfiedLinkError(
                "OntoLogos native library not found; build with `cargo build -p ontologos-jni --release` "
                        + "or set -Dontologos.native.path=/path/to/library");
    }
}
