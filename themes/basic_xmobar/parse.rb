#!/bin/env ruby

require 'json'
require 'socket'
SOCKET_FILE = "#{ENV["XDG_RUNTIME_DIR"]}/leftwm/current_state.sock"
puts SOCKET_FILE

$stdout.sync = true
Socket.unix(SOCKET_FILE) do |socket|
  while j = socket.gets
    hash =JSON.parse j

    actions = hash['desktop_names'].map do |n|
      if hash['active_desktop'].include?(n)
        # add active color
        "<action=xdotool key alt+#{n}><fc=#FF0000>  [#{n}]  </fc></action>"
      elsif hash['viewports'].include?(n)
        # add inactive color
        "<action=xdotool key alt+#{n}><fc=#0000FF>  (#{n})  </fc></action>"
      else
        "<action=xdotool key alt+#{n}>   #{n}   </action>"
      end
      #$stdout.puts "<action='xdotool key alt+#{n}'>#{n}</action>"

    end.join('')

    $stdout.puts actions
  end
end

