

Config { font = "-misc-fixed-*-*-*-*-13-*-*-*-*-*-*-*"
    , bgColor = "black"
    , fgColor = "grey"
    , position = Top
    , lowerOnStart = True
    , allDesktops = True
    , pickBroadest = False
    , overrideRedirect = False
    , commands = [    Run Weather "KSHV" ["-t"," <tempF>F","-L","64","-H","77","--normal","green","--high","red","--low","lightblue"] 36000
                    , Run Network "eno1" ["-L","0","-H","32","--normal","green","--high","red"] 10
                    , Run MultiCpu ["-L","15","-H","50","--normal","green","--high","red"] 10
                    , Run Memory [] 10
                    , Run Swap [] 10
                    , Run TopProc [] 10
                    , Run Date "%a %b %_d %Y %H:%M:%S" "date" 10
                    , Run UnsafeStdinReader
    ]
    , sepChar = "%"
    , alignSep = "}{"
    , template = "%UnsafeStdinReader% }{ %top% | %multicpu% | %memory% * %swap% | %eno1% | %date% | %KSHV%"
}
