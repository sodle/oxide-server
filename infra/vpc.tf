resource "aws_vpc" "vpc" {
  tags                 = { Name = "oxide" }
  cidr_block           = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true
}

resource "aws_subnet" "private_a" {
  vpc_id            = aws_vpc.vpc.id
  cidr_block        = "10.0.128.0/26"
  availability_zone = "${var.aws_region}a"
  tags              = { Name = "Private A" }
}

resource "aws_subnet" "private_b" {
  vpc_id            = aws_vpc.vpc.id
  cidr_block        = "10.0.192.0/26"
  availability_zone = "${var.aws_region}b"
  tags              = { Name = "Private B" }
}

resource "aws_vpc_endpoint" "ecr_dkr" {
  service_name        = "com.amazonaws.${var.aws_region}.ecr.dkr"
  vpc_id              = aws_vpc.vpc.id
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id, aws_subnet.private_b.id]
  private_dns_enabled = true
  security_group_ids  = [aws_security_group.endpoints.id]
}

resource "aws_vpc_endpoint" "ecr_api" {
  service_name        = "com.amazonaws.${var.aws_region}.ecr.api"
  vpc_id              = aws_vpc.vpc.id
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id, aws_subnet.private_b.id]
  private_dns_enabled = true
  security_group_ids  = [aws_security_group.endpoints.id]
}

resource "aws_vpc_endpoint" "ssm" {
  service_name        = "com.amazonaws.${var.aws_region}.ssm"
  vpc_id              = aws_vpc.vpc.id
  vpc_endpoint_type   = "Interface"
  subnet_ids          = [aws_subnet.private_a.id, aws_subnet.private_b.id]
  private_dns_enabled = true
  security_group_ids  = [aws_security_group.endpoints.id]
}

resource "aws_vpc_endpoint" "dynamodb" {
  service_name    = "com.amazonaws.${var.aws_region}.dynamodb"
  vpc_id          = aws_vpc.vpc.id
  route_table_ids = [aws_route_table.private_a.id, aws_route_table.private_b.id]
}

resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.vpc.id
}


resource "aws_subnet" "public_a" {
  vpc_id            = aws_vpc.vpc.id
  cidr_block        = "10.0.0.0/26"
  availability_zone = "${var.aws_region}a"
  tags              = { Name = "Private A" }
}

resource "aws_subnet" "public_b" {
  vpc_id            = aws_vpc.vpc.id
  cidr_block        = "10.0.64.0/26"
  availability_zone = "${var.aws_region}b"
  tags              = { Name = "Private B" }
}

resource "aws_eip" "nat_a" {}

resource "aws_nat_gateway" "nat_a" {
  subnet_id     = aws_subnet.public_a.id
  allocation_id = aws_eip.nat_a.allocation_id
}

resource "aws_eip" "nat_b" {}

resource "aws_nat_gateway" "nat_b" {
  subnet_id     = aws_subnet.public_b.id
  allocation_id = aws_eip.nat_b.allocation_id
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.vpc.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }
}

resource "aws_route_table_association" "public_a" {
  route_table_id = aws_route_table.public.id
  subnet_id      = aws_subnet.public_a.id
}

resource "aws_route_table_association" "public_b" {
  route_table_id = aws_route_table.public.id
  subnet_id      = aws_subnet.public_b.id
}

resource "aws_route_table" "private_a" {
  vpc_id = aws_vpc.vpc.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.nat_a.id
  }
}

resource "aws_route_table" "private_b" {
  vpc_id = aws_vpc.vpc.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.nat_b.id
  }
}

resource "aws_route_table_association" "private_a" {
  route_table_id = aws_route_table.private_a.id
  subnet_id      = aws_subnet.private_a.id
}

resource "aws_route_table_association" "private_b" {
  route_table_id = aws_route_table.private_b.id
  subnet_id      = aws_subnet.private_b.id
}

resource "aws_security_group" "endpoints" {
  name   = "VPC Endpoint SG"
  vpc_id = aws_vpc.vpc.id
  ingress {
    from_port   = 0
    to_port     = 0
    protocol    = "ALL"
    cidr_blocks = [aws_vpc.vpc.cidr_block]
  }
}
