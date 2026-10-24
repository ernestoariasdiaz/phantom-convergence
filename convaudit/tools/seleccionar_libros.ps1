# Copia a una carpeta nueva SOLO los libros que alimentaron ANIMUS en febrero 2026.
# Un archivo por libro: si hay versión en inglés y en español, toma el primero que encuentre
# en el orden de la lista. No mueve ni borra nada en la carpeta original.
#
# Uso:
#   .\tools\seleccionar_libros.ps1 -Origen "C:\ruta\pdfs" -Destino "C:\ruta\libros_animus"
#   (agrega -Simular para ver qué haría sin copiar nada)

param(
    [Parameter(Mandatory=$true)][string]$Origen,
    [Parameter(Mandatory=$true)][string]$Destino,
    [switch]$Simular
)

# Cada libro: lista de patrones en orden de preferencia (nombres vistos en el log de febrero primero)
$libros = [ordered]@{
  "Biblia"                 = @("Bible_complete*", "Biblia_completa*", "83800_spa*", "*biblia*", "*bible*")
  "Trump - Negociacion"    = @("*arte-de-la-negociacion*", "*art of the deal*", "*trump*")
  "Marco Aurelio"          = @("*Meditaciones*Marco Aurelio*", "*Meditations*", "*Marco Aurelio*")
  "Tao Te Ching"           = @("Tao Te Ching*", "*tao*te*ching*")
  "Sun Tzu"                = @("El Arte de la Guerra Sun Tzu*", "Arte de la Guerra*", "El Arte de la Guerra*", "*art of war*")
  "Kuhn"                   = @("Structure of Scientific Revolutions*", "Kuhn-SSR*", "*kuhn*")
  "Polya"                  = @("Como resolver problemas Polya*", "How to Solve It*", "George_Polya*", "*polya*")
  "Dawkins"                = @("El Gen Egoista*", "DAUKINS*", "*selfish gene*", "*dawkins*")
  "GEB"                    = @("GEBen*", "*godel*escher*bach*")
  "Penrose"                = @("lanueva*", "*penrose*", "*emperor*new mind*", "*nueva mente del emperador*")
  "Domingos"               = @("*Master Algorithm*", "*Algorithm_ How the Quest*", "*domingos*")
  "Petzold"                = @("code-charles-petzold*", "*petzold*")
  "Scott"                  = @("J Clark Scott*", "*But How Do It Know*")
  "Tanenbaum - OS"         = @("Operating Systems Design and Implementation*", "Operating-Systems-Design*", "*operating*systems*")
  "Tanenbaum - Redes"      = @("Computer Networks*", "Computer-Networks*", "*redes*computadoras*")
  "K&R"                    = @("C_Book_2nd*", "*kernighan*", "*c programming language*")
  "Pragmatic Programmer"   = @("*pragmatic*", "*pragmatico*")
  "Unix"                   = @("UNIX_A_History*", "*unix*")
  "Kleppmann"              = @("*kleppmann*", "*data-intensive*", "*data intensive*")
  "Rust"                   = @("The Rust Programming Language*", "rust_2018*", "*rust*")
  "Taleb"                  = @("Antifragile*", "Antifragil*", "*taleb*")
  "Meadows"                = @("Thinking in Systems*", "Meadows-2008*", "Pensar en sistemas*")
  "Axelrod"                = @("Evolution of Cooperation*", "*axelrod*")
  "Diamond"                = @("Guns_Germs_Steel*", "Armas Germenes y Acero*", "*guns*germs*")
  "Shirer"                 = @("Rise_Fall_Third_Reich*", "*third reich*", "*tercer reich*")
  "Acemoglu"               = @("Why_Nations_Fail*", "*why nations fail*", "*fracasan los paises*", "*acemoglu*")
  "Graeber"                = @("Debt_First_5000_Years*", "*Debt*First*5*000*", "*graeber*", "*deuda*5000*")
  "Klein"                  = @("Shock_Doctrine*", "Naomi Klein*", "*shock*")
  "Mitchell"               = @("Complexity_Guided_Tour*", "*complexity*", "*mitchell*")
  "Fukuyama"               = @("*fukuyama*", "*origins*of*political*order*", "*origenes*del*orden*politico*")
}

$pdfs = Get-ChildItem -Path $Origen -Filter *.pdf -File -Recurse
if (-not $Simular) { New-Item -ItemType Directory -Force -Path $Destino | Out-Null }

$usados = @{}
$faltan = @()
foreach ($libro in $libros.Keys) {
  $elegido = $null
  foreach ($pat in $libros[$libro]) {
    $elegido = $pdfs | Where-Object { $_.Name -like $pat -and -not $usados.ContainsKey($_.FullName) } | Select-Object -First 1
    if ($elegido) { break }
  }
  if ($elegido) {
    $usados[$elegido.FullName] = $libro
    Write-Host ("OK     {0,-22} <- {1}" -f $libro, $elegido.Name) -ForegroundColor Green
    if (-not $Simular) { Copy-Item $elegido.FullName -Destination $Destino }
  } else {
    $faltan += $libro
    Write-Host ("FALTA  {0}" -f $libro) -ForegroundColor Yellow
  }
}

Write-Host ""
Write-Host "PDFs que NO se copiaron (revisa que ninguno sea un libro de ANIMUS con otro nombre):" -ForegroundColor Cyan
$pdfs | Where-Object { -not $usados.ContainsKey($_.FullName) } | ForEach-Object { Write-Host "  - $($_.Name)" }
Write-Host ""
Write-Host ("Copiados: {0} de {1} libros.  Faltan: {2}" -f $usados.Count, $libros.Count, ($faltan -join ", "))
