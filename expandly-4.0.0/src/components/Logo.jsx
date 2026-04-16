import logoMarkup from '../assets/logo.svg?raw'

const inlineLogoMarkup = logoMarkup.replace(
  '<svg ',
  '<svg class="w-full h-full" ',
)

export default function Logo({ className = '', title = 'Expandly' }) {
  return (
    <span
      className={`inline-block text-white ${className}`.trim()}
      aria-label={title}
      role="img"
      dangerouslySetInnerHTML={{ __html: inlineLogoMarkup }}
    />
  )
}
