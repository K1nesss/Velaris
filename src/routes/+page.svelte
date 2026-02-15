<script lang="ts" module>
  import { redirect } from '@sveltejs/kit';

  export const load = () => {
    throw redirect(302, '/dashboard');
  };
</script>

