resource "aws_lb" "oxide" {
  subnets            = [aws_subnet.public_a.id, aws_subnet.public_b.id]
  load_balancer_type = "application"
  security_groups    = [aws_security_group.oxide_lb.id]
}

resource "aws_alb_target_group" "oxide" {
  target_type = "ip"
  port        = 3000
  protocol    = "HTTP"
  vpc_id      = aws_vpc.vpc.id
  health_check {
    enabled = true
    path    = "/health"
  }
}

resource "aws_alb_listener" "oxide" {
  load_balancer_arn = aws_lb.oxide.arn
  protocol          = "HTTP"
  port              = 80
  default_action {
    type = "forward"
    forward {
      target_group {
        arn = aws_alb_target_group.oxide.arn
      }
    }
  }
}

data "aws_ec2_managed_prefix_list" "cloudfront" {
  name = "com.amazonaws.global.cloudfront.origin-facing"
}

resource "aws_security_group" "oxide_lb" {
  name   = "Oxide LB Public"
  vpc_id = aws_vpc.vpc.id
  ingress {
    prefix_list_ids = [data.aws_ec2_managed_prefix_list.cloudfront.id]
    from_port       = 80
    to_port         = 80
    protocol        = "TCP"
  }
  egress {
    cidr_blocks = [aws_vpc.vpc.cidr_block]
    from_port   = 3000
    to_port     = 3000
    protocol    = "TCP"
  }
}
