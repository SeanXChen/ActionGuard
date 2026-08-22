# Generate a simple AgentGuard icon (shield) using System.Drawing.
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$size = 1024
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.Clear([System.Drawing.Color]::Transparent)

# Dark rounded background
$bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Point(0,0)),
    (New-Object System.Drawing.Point($size,$size)),
    [System.Drawing.Color]::FromArgb(255,15,23,42),
    [System.Drawing.Color]::FromArgb(255,30,41,59))
$g.FillRectangle($bgBrush, 0, 0, $size, $size)

function Pt($x, $y) {
    New-Object System.Drawing.PointF([single]$x, [single]$y)
}

# Shield body
$shield = New-Object System.Drawing.Drawing2D.GraphicsPath
$cx = $size / 2
$topY = $size * 0.18
$botY = $size * 0.78
$halfW = $size * 0.30
$points = @(
    (Pt ($cx - $halfW) $topY),
    (Pt ($cx + $halfW) $topY),
    (Pt ($cx + $halfW) $botY),
    (Pt $cx ($size * 0.90)),
    (Pt ($cx - $halfW) $botY)
)
$shield.AddLines([System.Drawing.PointF[]]$points)
$shield.CloseFigure()

$grad = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Point(0,0)),
    (New-Object System.Drawing.Point(0,$size)),
    [System.Drawing.Color]::FromArgb(255,34,197,94),
    [System.Drawing.Color]::FromArgb(255,16,120,68))
$g.FillPath($grad, $shield)

$pen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255,74,222,128), 16)
$g.DrawPath($pen, $shield)

# Checkmark
$check = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 60)
$check.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$check.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
$g.DrawLine($check, (Pt ($cx - $size*0.18) ($size*0.50)), (Pt ($cx - $size*0.02) ($size*0.64)))
$g.DrawLine($check, (Pt ($cx - $size*0.02) ($size*0.64)), (Pt ($cx + $size*0.20) ($size*0.34)))

$g.Dispose()
$out = Join-Path $PSScriptRoot "app-icon.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Host "Icon written to $out"
