resource "aws_ecr_repository" "alloy" {
  name                 = "alloy"
  image_tag_mutability = "IMMUTABLE"
  force_delete         = true

  image_scanning_configuration {
    scan_on_push = true
  }
}

resource "random_id" "alloy_img_tag" {
  keepers = {
    dockerfile_hash = filebase64sha512("${path.module}/../monitoring/alloy/Dockerfile")
    config_hash     = filebase64sha512("${path.module}/../monitoring/alloy/config.alloy")
  }
  byte_length = 2
}

resource "null_resource" "alloy_img_build" {
  triggers = {
    docker_img_tag = random_id.alloy_img_tag.hex
    repo_id        = aws_ecr_repository.alloy.id
  }

  provisioner "local-exec" {
    command = <<-EOT
      aws ecr get-login-password --region ${var.aws_region} | docker login --username AWS --password-stdin ${aws_ecr_repository.alloy.repository_url}
      docker build --platform=linux/arm64 -t ${aws_ecr_repository.alloy.repository_url}:${random_id.alloy_img_tag.hex} ${path.module}/../monitoring/alloy
      docker push ${aws_ecr_repository.alloy.repository_url}:${random_id.alloy_img_tag.hex}
    EOT
  }
}

data "aws_ssm_parameter" "alloy_endpoint" {
  name = "/oxide/alloy-endpoint"
}
data "aws_ssm_parameter" "alloy_username" {
  name = "/oxide/alloy-username"
}
data "aws_ssm_parameter" "alloy_endpoint_loki" {
  name = "/oxide/alloy-endpoint/loki"
}
data "aws_ssm_parameter" "alloy_username_loki" {
  name = "/oxide/alloy-username/loki"
}
data "aws_ssm_parameter" "alloy_token" {
  name = "/oxide/alloy-token"
}
