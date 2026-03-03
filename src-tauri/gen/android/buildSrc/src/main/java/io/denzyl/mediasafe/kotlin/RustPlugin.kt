import com.android.build.api.dsl.ApplicationExtension
import org.gradle.api.DefaultTask
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.configure
import org.gradle.kotlin.dsl.get

const val TASK_GROUP = "rust"

open class Config {
    lateinit var rootDirRel: String
}

open class RustPlugin : Plugin<Project> {
    private lateinit var config: Config

    override fun apply(project: Project) = with(project) {
        config = extensions.create("rust", Config::class.java)

        val defaultAbiList = listOf("arm64-v8a", "x86_64")
        val defaultArchList = listOf("arm64", "x86_64")
        val defaultTargetsList = listOf("aarch64", "x86_64")

        val abiList = (findProperty("abiList") as? String)?.split(',') ?: defaultAbiList
        val archList = (findProperty("archList") as? String)?.split(',') ?: defaultArchList
        val targetsList = (findProperty("targetList") as? String)?.split(',') ?: defaultTargetsList

        // Find indices of architectures we want to support (64-bit only)
        val supportedIndices = mutableListOf<Int>()
        for (i in archList.indices) {
            if (archList[i] == "arm64" || archList[i] == "x86_64") {
                supportedIndices.add(i)
            }
        }

        val finalAbiList = supportedIndices.map { abiList[it] }
        val finalArchList = supportedIndices.map { archList[it] }
        val finalTargetsList = supportedIndices.map { targetsList[it] }

        extensions.configure<ApplicationExtension> {
            @Suppress("UnstableApiUsage")
            flavorDimensions.add("abi")
            productFlavors {
                create("universal") {
                    dimension = "abi"
                    ndk {
                        abiFilters += finalAbiList
                    }
                }
                finalArchList.forEachIndexed { index, arch ->
                    create(arch) {
                        dimension = "abi"
                        ndk {
                            abiFilters.add(finalAbiList[index])
                        }
                    }
                }
            }
        }

        afterEvaluate {
            for (profile in listOf("debug", "release")) {
                val profileCapitalized = profile.replaceFirstChar { it.uppercase() }
                val buildTask = tasks.maybeCreate(
                    "rustBuildUniversal$profileCapitalized",
                    DefaultTask::class.java
                ).apply {
                    group = TASK_GROUP
                    description = "Build dynamic library in $profile mode for all targets"
                }

                tasks["mergeUniversal${profileCapitalized}JniLibFolders"].dependsOn(buildTask)

                for (targetPair in finalTargetsList.withIndex()) {
                    val targetName = targetPair.value
                    val targetArch = finalArchList[targetPair.index]
                    val targetArchCapitalized = targetArch.replaceFirstChar { it.uppercase() }
                    val targetBuildTask = project.tasks.maybeCreate(
                        "rustBuild$targetArchCapitalized$profileCapitalized",
                        BuildTask::class.java
                    ).apply {
                        group = TASK_GROUP
                        description = "Build dynamic library in $profile mode for $targetArch"
                        rootDirRel = config.rootDirRel
                        target = targetName
                        release = profile == "release"
                    }

                    buildTask.dependsOn(targetBuildTask)
                    tasks["merge$targetArchCapitalized${profileCapitalized}JniLibFolders"].dependsOn(
                        targetBuildTask
                    )
                }
            }
        }
    }
}