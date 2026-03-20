data "aws_iam_policy_document" "task_policy" {
  statement {
    actions   = ["dynamodb:PutItem", "dynamodb:GetItem"]
    resources = [aws_dynamodb_table.url_table.arn]
  }

  statement {
    actions   = ["logs:CreateLogGroup"]
    resources = ["*"]
  }
}

resource "aws_iam_policy" "task_policy" {
  policy = data.aws_iam_policy_document.task_policy.json
  name   = "oxide_task_policy"
}

data "aws_iam_policy_document" "service_role_trustee" {
  statement {
    principals {
      identifiers = ["ecs-tasks.amazonaws.com"]
      type        = "Service"
    }
    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "service_role" {
  assume_role_policy  = data.aws_iam_policy_document.service_role_trustee.json
  managed_policy_arns = ["arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"]
  name                = "oxide_service_role"
}

resource "aws_iam_role" "task_role" {
  assume_role_policy  = data.aws_iam_policy_document.service_role_trustee.json
  managed_policy_arns = [aws_iam_policy.task_policy.arn]
  name                = "oxide_task_role"
}

resource "aws_ecs_task_definition" "oxide_server" {
  family = "oxide_server"
  container_definitions = jsonencode([{
    name      = "oxide_server"
    image     = "${aws_ecr_repository.oxide_server.repository_url}:${null_resource.docker_img_build.triggers.docker_img_tag}"
    essential = true
    portMappings = [{
      hostPort      = 3000,
      containerPort = 3000,
    }]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-create-group" : "true",
        "awslogs-group" : "oxide",
        "awslogs-region" : var.aws_region,
        "awslogs-stream-prefix" : "oxide"
      }
    }
    environment = [
      {
        name  = "DYNAMODB_TABLE_NAME"
        value = "oxide-urls"
      },
      {
        name  = "RUST_BACKTRACE"
        value = "1"
      },
      {
        name  = "RUST_LOG"
        value = "debug"
      }
    ]
  }])
  cpu                      = 1024
  memory                   = 2048
  requires_compatibilities = ["FARGATE"]
  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture        = "ARM64"
  }
  network_mode       = "awsvpc"
  execution_role_arn = aws_iam_role.service_role.arn
  task_role_arn      = aws_iam_role.task_role.arn
}

resource "aws_ecs_cluster" "oxide_server" {
  name = "oxide_server"
}

resource "aws_ecs_service" "oxide_server" {
  name            = "oxide_server"
  cluster         = aws_ecs_cluster.oxide_server.id
  task_definition = aws_ecs_task_definition.oxide_server.id
  network_configuration {
    subnets         = [aws_subnet.private_a.id, aws_subnet.private_b.id]
    security_groups = [aws_security_group.oxide_server.id]
  }
  desired_count = 1
  launch_type   = "FARGATE"
  load_balancer {
    container_name   = "oxide_server"
    container_port   = 3000
    target_group_arn = aws_alb_target_group.oxide.arn
  }
}

resource "aws_security_group" "oxide_server" {
  name   = "oxide_lb"
  vpc_id = aws_vpc.vpc.id
  ingress {
    cidr_blocks = [aws_vpc.vpc.cidr_block]
    to_port     = 3000
    from_port   = 3000
    protocol    = "TCP"
  }
  egress {
    cidr_blocks = ["0.0.0.0/0"]
    protocol    = "ALL"
    from_port   = 0
    to_port     = 0
  }
}
