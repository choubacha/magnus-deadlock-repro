# frozen_string_literal: true

require_relative './spec_helper'

RSpec.describe 'Deadlock' do
  it 'sometimes deadlocks when creating ruby exts inside threads' do
    threads = Array.new(100).map do
      Thread.new do
        point = Point.new(1, 2)
        expect(point.x).to eq 1
        expect(point.y).to eq 2
      end
    end

    threads.each(&:join)
  end
end
