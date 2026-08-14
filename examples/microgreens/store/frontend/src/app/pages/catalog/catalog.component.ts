import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { AuthService } from '../../core/services/auth.service';
import { CartService } from '../../core/services/cart.service';
import { Product } from '../../core/models/models';
import { ProductService } from '../../core/services/product.service';

@Component({
  selector: 'app-catalog',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './catalog.component.html',
})
export class CatalogComponent implements OnInit {
  products = signal<Product[]>([]);
  quantities = new Map<number, number>();
  error = signal<string | null>(null);
  message = signal<string | null>(null);

  constructor(
    private productService: ProductService,
    private cartService: CartService,
    public auth: AuthService,
    private router: Router,
  ) {}

  ngOnInit(): void {
    this.load();
  }

  load(): void {
    this.productService.list().subscribe({
      next: (products) => this.products.set(products),
      error: () => this.error.set('Could not load products.'),
    });
  }

  quantityFor(productId: number): number {
    return this.quantities.get(productId) ?? 1;
  }

  setQuantity(productId: number, value: string): void {
    this.quantities.set(productId, Math.max(1, Number(value) || 1));
  }

  addToCart(product: Product): void {
    this.error.set(null);
    this.message.set(null);

    if (!this.auth.isLoggedIn()) {
      this.router.navigate(['/login']);
      return;
    }

    const quantity = this.quantityFor(product.id);
    this.cartService.addToCart(product.id, quantity).subscribe({
      next: () => this.message.set(`Added ${quantity} tray(s) of ${product.variety} to your cart.`),
      error: (err) => this.error.set(err?.error?.error ?? 'Could not add to cart.'),
    });
  }
}
