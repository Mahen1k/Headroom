import { CommonModule } from '@angular/common';
import { Component, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { AdminService } from '../../core/services/admin.service';
import { ProductService } from '../../core/services/product.service';

@Component({
  selector: 'app-admin',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './admin.component.html',
})
export class AdminComponent implements OnInit {
  varieties = signal<string[]>([]);
  message = signal<string | null>(null);
  error = signal<string | null>(null);

  batch = {
    variety: '',
    sow_date: '',
    quantity_trays: 1,
    price_per_tray: 4.5,
  };

  inspect = {
    batch_id: 0,
    height_cm: 5,
    mold: false,
    color_uniform: true,
    stem_ok: true,
    root_mat_ok: true,
  };

  constructor(private adminService: AdminService, private productService: ProductService) {}

  ngOnInit(): void {
    this.productService.varieties().subscribe({
      next: (varieties) => {
        this.varieties.set(varieties);
        this.batch.variety = varieties[0] ?? '';
      },
    });
  }

  submitBatch(): void {
    this.message.set(null);
    this.error.set(null);
    this.adminService.addBatch(this.batch).subscribe({
      next: (res) => {
        this.message.set(`Batch #${res.id} added.`);
        this.inspect.batch_id = res.id;
      },
      error: (err) => this.error.set(err?.error?.error ?? 'Could not add batch.'),
    });
  }

  submitInspect(): void {
    this.message.set(null);
    this.error.set(null);
    const { batch_id, ...body } = this.inspect;
    this.adminService.inspectBatch(batch_id, body).subscribe({
      next: () => this.message.set(`Batch #${batch_id} quality check recorded.`),
      error: (err) => this.error.set(err?.error?.error ?? 'Could not record quality check.'),
    });
  }
}
