param(
    [string]$StateFile
)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

function New-Text([int[]]$CodePoints) {
    return -join ($CodePoints | ForEach-Object { [char]$_ })
}

function Move-ToBottomCenter($TargetForm) {
    $screen = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
    $x = [int](($screen.Width - $TargetForm.Width) / 2) + $screen.Left
    $y = [Math]::Max(12, $screen.Bottom - $TargetForm.Height - 26)
    $TargetForm.Location = New-Object System.Drawing.Point($x, $y)
}

function New-CapsulePath([float]$X, [float]$Y, [float]$Width, [float]$Height) {
    $radius = $Height
    $diameter = $radius
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($X, $Y, $diameter, $diameter, 90, 180)
    $path.AddArc($X + $Width - $diameter, $Y, $diameter, $diameter, 270, 180)
    $path.CloseFigure()
    return $path
}

function Snap-Float([double]$Value, [double]$Step) {
    if ($Step -le 0) { return [double]$Value }
    return [Math]::Round($Value / $Step) * $Step
}

function New-RoundedRectPath([float]$X, [float]$Y, [float]$Width, [float]$Height, [float]$Radius) {
    $diameter = [Math]::Min($Radius * 2.0, [Math]::Min($Width, $Height))
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($X, $Y, $diameter, $diameter, 180, 90)
    $path.AddArc($X + $Width - $diameter, $Y, $diameter, $diameter, 270, 90)
    $path.AddArc($X + $Width - $diameter, $Y + $Height - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($X, $Y + $Height - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    return $path
}

function Format-Elapsed([long]$StartedAtMs) {
    if ($StartedAtMs -le 0) {
        return '0:00'
    }

    $elapsedMs = [Math]::Max(0, [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() - $StartedAtMs)
    $elapsedSeconds = [int][Math]::Floor($elapsedMs / 1000.0)
    $minutes = [int][Math]::Floor($elapsedSeconds / 60)
    $seconds = $elapsedSeconds % 60
    return '{0}:{1:d2}' -f $minutes, $seconds
}

function Draw-MicButton($Graphics, [float]$CenterX, [float]$CenterY) {
    if ($script:mode -eq 'listening') {
        foreach ($phaseOffset in @(0.0, 0.5)) {
            $progress = (($script:phase * 0.022) + $phaseOffset) % 1.0
            $ringSize = 40 + ($progress * 18)
            $ringAlpha = [int](90 * (1.0 - $progress))
            if ($ringAlpha -gt 0) {
                $ringPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb($ringAlpha, 218, 214, 207), 2)
                $Graphics.DrawEllipse($ringPen, ($CenterX - ($ringSize / 2)), ($CenterY - ($ringSize / 2)), $ringSize, $ringSize)
            }
        }

        $innerRingPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(165, 226, 222, 215), 1.5)
        $Graphics.DrawEllipse($innerRingPen, ($CenterX - 21.5), ($CenterY - 21.5), 43, 43)
    } elseif ($script:mode -eq 'processing') {
        foreach ($phaseOffset in @(0.0, 0.4, 0.8)) {
            $progress = (($script:phase * 0.018) + $phaseOffset) % 1.0
            $ringSize = 42 + ($progress * 18)
            $ringAlpha = [int](78 * (1.0 - $progress))
            if ($ringAlpha -gt 0) {
                $ringPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb($ringAlpha, 232, 228, 220), 2)
                $Graphics.DrawEllipse($ringPen, ($CenterX - ($ringSize / 2)), ($CenterY - ($ringSize / 2)), $ringSize, $ringSize)
            }
        }

        $glow = (1.0 + [Math]::Sin($script:phase * 0.12)) / 2.0
        $coreAlpha = [int](135 + ($glow * 65))
        $coreSize = 9.5 + ($glow * 2.0)
        $innerRingAlpha = [int](80 + ($glow * 50))
        $innerRingPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb($innerRingAlpha, 244, 241, 236), 1.5)
        $Graphics.DrawEllipse($innerRingPen, ($CenterX - 13.5), ($CenterY - 13.5), 27, 27)
        $coreBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb($coreAlpha, 255, 255, 255))
        $Graphics.FillEllipse($coreBrush, ($CenterX - ($coreSize / 2)), ($CenterY - ($coreSize / 2)), $coreSize, $coreSize)
        return
    } else {
        $ringPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(225, 221, 214), 2)
        $Graphics.DrawEllipse($ringPen, ($CenterX - 24), ($CenterY - 24), 48, 48)
    }

    $fillBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(10, 10, 10))
    $Graphics.FillEllipse($fillBrush, ($CenterX - 19), ($CenterY - 19), 38, 38)

    $pen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2.2)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    $bodyPath = New-RoundedRectPath ($CenterX - 4.6) ($CenterY - 10.0) 9.2 12.2 4.6
    $Graphics.DrawPath($pen, $bodyPath)

    $supportPoints = @(
        (New-Object System.Drawing.PointF(($CenterX - 8.6), ($CenterY - 0.8))),
        (New-Object System.Drawing.PointF(($CenterX - 8.6), ($CenterY + 4.2))),
        (New-Object System.Drawing.PointF(($CenterX - 6.8), ($CenterY + 7.8))),
        (New-Object System.Drawing.PointF(($CenterX - 3.8), ($CenterY + 10.0))),
        (New-Object System.Drawing.PointF($CenterX, ($CenterY + 10.8))),
        (New-Object System.Drawing.PointF(($CenterX + 3.8), ($CenterY + 10.0))),
        (New-Object System.Drawing.PointF(($CenterX + 6.8), ($CenterY + 7.8))),
        (New-Object System.Drawing.PointF(($CenterX + 8.6), ($CenterY + 4.2))),
        (New-Object System.Drawing.PointF(($CenterX + 8.6), ($CenterY - 0.8)))
    )
    $Graphics.DrawLines($pen, $supportPoints)

    $stemPen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2.0)
    $stemPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $stemPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $Graphics.DrawLine($stemPen, $CenterX, ($CenterY + 10.8), $CenterX, ($CenterY + 16.6))
    $Graphics.DrawLine($stemPen, ($CenterX - 6.6), ($CenterY + 18.4), ($CenterX + 6.6), ($CenterY + 18.4))
}

$script:fontFamily = 'Microsoft YaHei UI'
$script:listeningText = New-Text @(0x76D1, 0x542C, 0x4E2D)
$script:processingText = New-Text @(0x8BC6, 0x522B, 0x4E2D)
$script:errorText = New-Text @(0x9EA6, 0x514B, 0x98CE, 0x6743, 0x9650, 0x88AB, 0x62D2, 0x7EDD)
$script:mode = 'idle'
$script:visible = $false
$script:phase = 0.0
$script:level = 0.0
$script:startedAtMs = 0
$script:lastVisible = $false
$script:processingMorph = 0.0
$script:clock = [System.Diagnostics.Stopwatch]::StartNew()
$script:lastTickMs = 0
$script:lastStatePollMs = 0

$transparency = [System.Drawing.Color]::FromArgb(255, 1, 0, 1)
$form = New-Object System.Windows.Forms.Form
$form.Text = 'Vibe Coding Voice Overlay'
$form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
$form.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
$form.ShowInTaskbar = $false
$form.TopMost = $true
$form.BackColor = $transparency
$form.TransparencyKey = $transparency
$form.ClientSize = New-Object System.Drawing.Size(430, 92)
$doubleBufferedProp = $form.GetType().GetProperty('DoubleBuffered', [System.Reflection.BindingFlags]'NonPublic,Instance')
if ($doubleBufferedProp) { $doubleBufferedProp.SetValue($form, $true, $null) }

Move-ToBottomCenter $form
$form.Hide()

$form.Add_Resize({
    Move-ToBottomCenter $form
})

$form.Add_Paint({
    param($sender, $e)

    $g = $e.Graphics
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear($transparency)
    $morph = $script:processingMorph
    $isProcessing = $script:mode -eq 'processing'
    $capsuleCenterX = $form.ClientSize.Width / 2.0
    $capsuleCenterY = 39.0
    $capsuleWidth = 406.0 + ((272.0 - 406.0) * $morph)
    $capsuleHeight = 58.0 + ((52.0 - 58.0) * $morph)
    $capsuleX = $capsuleCenterX - ($capsuleWidth / 2.0)
    $capsuleY = $capsuleCenterY - ($capsuleHeight / 2.0)

    # Snap to a half-pixel grid to stabilize AA during morph animation.
    $capsuleX = Snap-Float $capsuleX 0.5
    $capsuleY = Snap-Float $capsuleY 0.5
    $capsuleWidth = [Math]::Max(1.0, (Snap-Float $capsuleWidth 0.5))
    $capsuleHeight = [Math]::Max(1.0, (Snap-Float $capsuleHeight 0.5))
    $shadowX = $capsuleX + 2
    $shadowY = $capsuleY + 4

    $shadowPath = New-CapsulePath $shadowX $shadowY $capsuleWidth $capsuleHeight
    $shadowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(38, 0, 0, 0))
    $g.FillPath($shadowBrush, $shadowPath)

    $capsulePath = New-CapsulePath $capsuleX $capsuleY $capsuleWidth $capsuleHeight
    $capsuleBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(250, 250, 247))
    $g.FillPath($capsuleBrush, $capsulePath)
    # Softer 2-layer border to hide subpixel jaggies.
    $borderOuter = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(18, 0, 0, 0), 2)
    $borderOuter.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $borderInner = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(48, 0, 0, 0), 1)
    $borderInner.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $g.DrawPath($borderOuter, $capsulePath)
    $g.DrawPath($borderInner, $capsulePath)
    if (-not $isProcessing -and $morph -lt 0.35) {
        Draw-MicButton $g ($capsuleX + 32) $capsuleCenterY
    }

    switch ($script:mode) {
        'processing' {
            $font = New-Object System.Drawing.Font($script:fontFamily, 10.5, [System.Drawing.FontStyle]::Regular)
            $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(21, 20, 15))
            $textRect = New-Object System.Drawing.RectangleF($capsuleX, $capsuleY, $capsuleWidth, $capsuleHeight)
            $format = New-Object System.Drawing.StringFormat
            $format.Alignment = [System.Drawing.StringAlignment]::Center
            $format.LineAlignment = [System.Drawing.StringAlignment]::Center
            # Center the label within the capsule; keep enough height to avoid clipping.
            $g.DrawString($script:processingText, $font, $textBrush, $textRect, $format)

            $trackPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(24, 21, 20, 15), 1)
            $trackPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
            $trackPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
            $trackLeft = $capsuleX + (($capsuleWidth - 92) / 2)
            $trackRight = $trackLeft + 92
            $trackY = $capsuleCenterY + 14
            $g.DrawLine($trackPen, $trackLeft, $trackY, $trackRight, $trackY)

            $phase = (($script:phase * 0.024) % 1.0)
            $segmentWidth = 34
            $segmentCenterX = $trackLeft + (92 * $phase)
            $segmentLeft = $segmentCenterX - ($segmentWidth / 2)
            $segmentRight = $segmentCenterX + ($segmentWidth / 2)
            $segmentBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
                (New-Object System.Drawing.PointF($segmentLeft, $trackY)),
                (New-Object System.Drawing.PointF($segmentRight, $trackY)),
                [System.Drawing.Color]::FromArgb(0, 21, 20, 15),
                [System.Drawing.Color]::FromArgb(0, 21, 20, 15)
            )
            $segmentBlend = New-Object System.Drawing.Drawing2D.ColorBlend
            $segmentBlend.Colors = @(
                [System.Drawing.Color]::FromArgb(0, 21, 20, 15),
                [System.Drawing.Color]::FromArgb(150, 21, 20, 15),
                [System.Drawing.Color]::FromArgb(0, 21, 20, 15)
            )
            $segmentBlend.Positions = @(0.0, 0.5, 1.0)
            $segmentBrush.InterpolationColors = $segmentBlend
            $segmentPen = New-Object System.Drawing.Pen($segmentBrush, 2.2)
            $segmentPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
            $segmentPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
            $g.DrawLine($segmentPen, $segmentLeft, $trackY, $segmentRight, $trackY)
        }
        'error' {
            $font = New-Object System.Drawing.Font($script:fontFamily, 10.5, [System.Drawing.FontStyle]::Regular)
            $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(154, 42, 34))
            $g.DrawString($script:errorText, $font, $textBrush, ($capsuleX + 76), ($capsuleCenterY - 13))
        }
        default {
            $linePen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(18, 21, 20, 15), 1)
            $contentLeft = $capsuleX + 76
            $contentRight = $capsuleX + 276
            $g.DrawLine($linePen, $contentLeft, $capsuleCenterY, $contentRight, $capsuleCenterY)
            $wave = 1.4 + ($script:level * 10.0)
            for ($i = 0; $i -lt 28; $i++) {
                $speed = 0.75 + (($i % 5) * 0.18)
                $x = $contentLeft + ((($script:phase * $speed * 6.1) + ($i * 10.2)) % 200)
                $jitter = (1.5 + ($i % 3)) * $wave
                $y = $capsuleCenterY + ([Math]::Sin(($script:phase * 0.35) + ($i * 0.7)) * $jitter)
                $tone = (($script:phase * 0.55) + ($i * 0.5))
                $depth = [Math]::Max(0.0, [Math]::Min(1.0, 0.5 + ([Math]::Sin($tone) * 0.5)))
                $darkness = [Math]::Max(0.0, [Math]::Min(1.0, 0.20 + ($depth * 0.65) + ($script:level * 0.25)))
                $gray = [int](205 - ($darkness * 190))
                $gray = [Math]::Max(18, [Math]::Min(205, $gray))
                $size = 2.3 + (($i * 17) % 5) * 0.40 + ($darkness * 2.5) + ($script:level * 1.35)
                $opacity = [Math]::Max(55, [Math]::Min(255, 95 + ($darkness * 120)))
                $particleBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb([int]$opacity, $gray, $gray, $gray))
                $g.FillEllipse($particleBrush, [float]$x, [float]$y, [float]$size, [float]$size)
            }

            $font = New-Object System.Drawing.Font($script:fontFamily, 9.5, [System.Drawing.FontStyle]::Regular)
            $labelBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(90, 88, 82))
            $labelRect = New-Object System.Drawing.RectangleF(
                ($capsuleX + $capsuleWidth - 120),
                $capsuleY,
                108,
                $capsuleHeight
            )
            $labelFormat = New-Object System.Drawing.StringFormat
            $labelFormat.Alignment = [System.Drawing.StringAlignment]::Far
            $labelFormat.LineAlignment = [System.Drawing.StringAlignment]::Center
            $g.DrawString($script:listeningText, $font, $labelBrush, $labelRect, $labelFormat)
        }
    }
})

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 16
$timer.Add_Tick({
    $nowMs = [int]$script:clock.ElapsedMilliseconds
    $dtMs = $nowMs - $script:lastTickMs
    if ($dtMs -le 0) { $dtMs = 16 }
    if ($dtMs -gt 80) { $dtMs = 80 } # avoid huge jumps after stalls
    $script:lastTickMs = $nowMs

    # Keep the original "phase units" (roughly 1.0 per 33ms) so existing animation math stays consistent.
    $script:phase += ($dtMs / 33.0)

    # Throttle state polling (JSON read/parse) to reduce stutter.
    if ($nowMs - $script:lastStatePollMs -ge 80) {
        $script:lastStatePollMs = $nowMs
        if ($StateFile -and (Test-Path $StateFile)) {
            try {
                $raw = Get-Content -LiteralPath $StateFile -Raw -Encoding UTF8
                if (-not [string]::IsNullOrWhiteSpace($raw)) {
                    $state = $raw | ConvertFrom-Json
                    $script:visible = [bool]$state.visible
                    $script:mode = if ($state.mode) { [string]$state.mode } else { 'listening' }
                    $script:level = if ($null -ne $state.level) { [double]$state.level } else { 0.0 }
                    $script:startedAtMs = if ($null -ne $state.started_at_ms) { [long]$state.started_at_ms } else { 0 }
                }
            } catch {
            }
        } else {
            $script:visible = $false
            $script:mode = 'idle'
            $script:level = 0.0
            $script:startedAtMs = 0
        }
    }

    $targetMorph = if ($script:mode -eq 'processing') { 1.0 } else { 0.0 }
    # Time-based smoothing: stable speed at 30/60fps and less "micro-jitter".
    $alpha = 1.0 - [Math]::Exp(-([double]$dtMs / 120.0))
    $script:processingMorph += ($targetMorph - $script:processingMorph) * $alpha
    if ([Math]::Abs($targetMorph - $script:processingMorph) -lt 0.002) {
        $script:processingMorph = $targetMorph
    }

    if ($script:visible -and -not $script:lastVisible) {
        Move-ToBottomCenter $form
        $form.Show()
        $form.TopMost = $true
        $script:lastVisible = $true
    } elseif (-not $script:visible -and $script:lastVisible) {
        $form.Hide()
        $script:lastVisible = $false
    }

    if ($script:lastVisible) {
        $form.Invalidate()
    }
})

$timer.Start()
$appContext = New-Object System.Windows.Forms.ApplicationContext
$form.Add_FormClosed({
    $appContext.ExitThread()
})
[System.Windows.Forms.Application]::Run($appContext)
