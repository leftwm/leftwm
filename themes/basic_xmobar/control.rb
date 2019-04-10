#!/bin/env ruby

require 'json'
require 'pty'
require 'socket'
SOCKET_FILE = "#{ENV["XDG_RUNTIME_DIR"]}/leftwm/current_state.sock"
CURRENT_PATH = File.expand_path(File.dirname(__FILE__))
Process.setproctitle("xmobar_control_script")
$0 = "xmobar_control_script"

##########################################################################
# NOTE: we want to put a bar at the top of every workspace, but we are not
# guaranteed a 1 to 1 relationship with screen. think ultrawide monitors and 
# people that like one massive workspace across all monitors. because of this
# we cannot expect the bar to know where the workspaces are and where to dock.
# we must get that information from leftwm
###########################################################################

$bars = []

# starts a view for every viewport in the list.
# adds stdin to the list of bars for piping info into it.
def start_bars viewports
  $bars = viewports.map do |view|
    stdin = nil
    position = "Static { xpos = #{view['x']} , ypos = #{view['y']}, width = #{view['w']}, height = 15 }"
    PTY.spawn('xmobar', '-p', position, "#{CURRENT_PATH}/xmobar-config.hs") do |output, input, pid|
      stdin = input
    end
    stdin
  end 
end

def format_for_view ws_index, hash
  displayed_tags = hash['viewports'].map{|vp| vp['tags'] }.flatten.uniq
  viewports_tags = hash['viewports'][ws_index]['tags'] 

  text = hash['desktop_names'].each_with_index.map do |name, tag_index|

    #tag_is_focused = hash['active_desktop'].include?(name)
    is_displayed   = displayed_tags.include?( name )
    is_mine        = viewports_tags.include?( name )

    mode = :normal
    mode = :alt     if is_displayed && !is_mine
    mode = :focused if is_displayed && is_mine
      
    case mode
    when :focused
      "<action=`#{CURRENT_PATH}/change_to_tag #{ws_index} #{tag_index}`><fc=#FF0000>  #{name}  </fc></action>"
    when :alt
      "<action=`#{CURRENT_PATH}/change_to_tag #{ws_index} #{tag_index}`><fc=#FFAAAA>  #{name}  </fc></action>"
    else #normal
      "<action=`#{CURRENT_PATH}/change_to_tag #{ws_index} #{tag_index}`><fc=#FFFFFF>  #{name}  </fc></action>"
    end
      
    end.join('')
    text = text + "<fc=#555555>     #{ hash['window_title'] }</fc>"
    text
end



$stdout.sync = true
Socket.unix(SOCKET_FILE) do |socket|
  while j = socket.gets
    hash =JSON.parse j
    #make sure the bars are running
    start_bars(hash['viewports']) if $bars.empty?
    #pipe the current state to the bars
    $bars.each_with_index do |stdin, index|
      bar_text = format_for_view(index, hash)
      stdin.puts(bar_text)
    end
  end
end

