#!/usr/bin/env perl

use v5.22;
use warnings;
use utf8;

use RFI::Loader "./target/rfi/hello.dat", "../../target/debug/libhello.dylib";

say hello::hello("Alex");
