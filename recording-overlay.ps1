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
    } else {
        $ringPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(225, 221, 214), 2)
        $Graphics.DrawEllipse($ringPen, ($CenterX - 24), ($CenterY - 24), 48, 48)
    }

    $fillBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(10, 10, 10))
    $Graphics.FillEllipse($fillBrush, ($CenterX - 19), ($CenterY - 19), 38, 38)

    $pen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, 2.0)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

    $bodyRect = New-Object System.Drawing.RectangleF(($CenterX - 4.0), ($CenterY - 9.5), 8.0, 12.0)
    $Graphics.DrawArc($pen, $bodyRect, 0, 360)

    $archRect = New-Object System.Drawing.RectangleF(($CenterX - 9.0), ($CenterY - 2.0), 18.0, 12.0)
    $Graphics.DrawArc($pen, $archRect, 12, 156)

    $Graphics.DrawLine($pen, $CenterX, ($CenterY + 3.5), $CenterX, ($CenterY + 7.5))
    $Graphics.DrawLine($pen, ($CenterX - 3.5), ($CenterY + 8.2), ($CenterX + 3.5), ($CenterY + 8.2))
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

    $shadowPath = New-CapsulePath 12 14 406 58
    $shadowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(38, 0, 0, 0))
    $g.FillPath($shadowBrush, $shadowPath)

    $capsulePath = New-CapsulePath 10 10 406 58
    $capsuleBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(250, 250, 247))
    $borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(18, 0, 0, 0), 1)
    $g.FillPath($capsuleBrush, $capsulePath)
    $g.DrawPath($borderPen, $capsulePath)
    Draw-MicButton $g 42 39

    switch ($script:mode) {
        'processing' {
            $font = New-Object System.Drawing.Font($script:fontFamily, 10.5, [System.Drawing.FontStyle]::Regular)
            $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(21, 20, 15))
            $mutedBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(136, 133, 119))
            for ($i = 0; $i -lt 3; $i++) {
                $offset = [Math]::Sin($script:phase * 0.18 + $i * 0.8)
                $alpha = [int](110 + (100 * (($offset + 1.0) / 2.0)))
                $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb($alpha, 21, 20, 15))
                $g.FillEllipse($brush, 86 + ($i * 11), 33, 6, 6)
            }
            $g.DrawString($script:processingText, $font, $textBrush, 126, 26)
            $g.DrawString((Format-Elapsed $script:startedAtMs), $font, $mutedBrush, 348, 26)
        }
        'error' {
            $font = New-Object System.Drawing.Font($script:fontFamily, 10.5, [System.Drawing.FontStyle]::Regular)
            $textBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(154, 42, 34))
            $g.DrawString($script:errorText, $font, $textBrush, 86, 26)
        }
        default {
            $linePen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(18, 21, 20, 15), 1)
            $g.DrawLine($linePen, 86, 39, 286, 39)
            $wave = 1.4 + ($script:level * 10.0)
            for ($i = 0; $i -lt 28; $i++) {
                $speed = 0.75 + (($i % 5) * 0.18)
                $x = 86 + ((($script:phase * $speed * 6.1) + ($i * 10.2)) % 200)
                $jitter = (1.5 + ($i % 3)) * $wave
                $y = 39 + ([Math]::Sin(($script:phase * 0.35) + ($i * 0.7)) * $jitter)
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
            $g.DrawString($script:listeningText, $font, $labelBrush, 322, 27)
        }
    }
})

$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 33
$timer.Add_Tick({
    $script:phase += 1.0

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
