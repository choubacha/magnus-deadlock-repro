# frozen_string_literal: true

require 'bundler/gem_tasks'
require 'rb_sys/extensiontask'

GEMSPEC = Gem::Specification.load('deadlock.gemspec')

RbSys::ExtensionTask.new('deadlock') do |c|
  c.lib_dir = 'lib/deadlock'
end
