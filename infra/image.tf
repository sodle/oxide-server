resource "aws_ecr_repository" "oxide_server" {
  name                 = "oxide-server"
  image_tag_mutability = "IMMUTABLE"
  force_delete         = true

  image_scanning_configuration {
    scan_on_push = true
  }
}

locals {
  dockerfile_hash = filebase64sha512("${path.module}/../Dockerfile")
  src_dir_hash = sha512(join("", [
    for f in sort(fileset("${path.module}/../src", "**.rs")) : filesha512("${path.module}/../src/${f}")
  ]))
}

resource "random_id" "docker_img_tag" {
  keepers = {
    dockerfile_hash = local.dockerfile_hash
    src_dir_hash    = local.src_dir_hash
  }
  byte_length = 2
}

resource "null_resource" "docker_img_build" {
  triggers = {
    docker_img_tag = random_id.docker_img_tag.hex
    repo_id        = aws_ecr_repository.oxide_server.id
  }

  provisioner "local-exec" {
    command = <<-EOT
      aws ecr get-login-password --region ${var.aws_region} | docker login --username AWS --password-stdin ${aws_ecr_repository.oxide_server.repository_url}
      docker build --platform=linux/arm64 -t ${aws_ecr_repository.oxide_server.repository_url}:${random_id.docker_img_tag.hex} ${path.module}/..
      docker push ${aws_ecr_repository.oxide_server.repository_url}:${random_id.docker_img_tag.hex}
    EOT
  }
}
