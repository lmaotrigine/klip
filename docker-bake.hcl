# in CI, this will be the short sha of the commit
# we expect this to be unset everywhere else so it just defaults to "latest"
# which is a sensible behaviour
variable "TAG" {
  default = "latest"
}

# this allows pinning
# tags that look like a semver aren't cleaned up by lifecycle policies
variable "RELEASE" {
  default = ""
}

# override this to include registry
variable "IMAGE_NAME" {
  default = "klip"
}

group "default" {
  targets = ["klip"]
}

target "klip" {
  tags = ["${IMAGE_NAME}:${TAG}", "${IMAGE_NAME}:latest", notequal(RELEASE, "") ? "${IMAGE_NAME}:${RELEASE}" : ""]
}
